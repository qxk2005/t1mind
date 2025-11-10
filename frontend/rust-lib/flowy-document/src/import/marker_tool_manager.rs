use flowy_error::{FlowyError, FlowyResult};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Marker 工具管理器
/// 
/// 负责从应用包中查找 Marker 工具，验证其可用性，并缓存路径以提高性能。
/// 支持 macOS 和 Windows 平台。
#[derive(Debug)]
pub struct MarkerToolManager {
    /// 缓存的 Marker 工具路径
    marker_path: Mutex<Option<PathBuf>>,
    /// 是否已经尝试过查找
    searched: Mutex<bool>,
}

impl MarkerToolManager {
    /// 创建新的 Marker 工具管理器
    /// 
    /// 管理器在首次调用 `get_marker_path()` 时会自动查找 Marker 工具。
    pub fn new() -> Self {
        Self {
            marker_path: Mutex::new(None),
            searched: Mutex::new(false),
        }
    }

    /// 从应用包中查找 Marker 工具
    /// 
    /// 查找策略：
    /// - macOS: 在 `AppFlowy.app/Contents/Resources/marker/` 目录中查找
    /// - Windows: 在 `AppFlowy/Resources/marker/` 目录中查找
    /// 
    /// 返回找到的 Marker 工具路径，如果未找到则返回错误。
    pub fn find_marker_in_bundle(&self) -> FlowyResult<PathBuf> {
        // 获取当前可执行文件路径
        let exe_path = std::env::current_exe().map_err(|e| {
            FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("无法获取当前可执行文件路径: {}", e),
            )
        })?;

        debug!("当前可执行文件路径: {}", exe_path.display());

        // 根据平台确定应用包资源目录
        let marker_path = self.resolve_bundle_marker_path(&exe_path)?;

        // 验证 Marker 工具是否存在且可执行
        self.verify_marker(&marker_path)?;

        info!("成功找到 Marker 工具: {}", marker_path.display());
        Ok(marker_path)
    }

    /// 解析应用包中的 Marker 工具路径
    /// 
    /// 基于当前可执行文件路径，推断应用包资源目录，并构建 Marker 工具路径。
    fn resolve_bundle_marker_path(&self, exe_path: &Path) -> FlowyResult<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            self.resolve_macos_bundle_path(exe_path)
        }

        #[cfg(target_os = "windows")]
        {
            self.resolve_windows_bundle_path(exe_path)
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Err(FlowyError::new(
                flowy_error::ErrorCode::NotSupportYet,
                format!("当前平台 {} 不支持 Marker 工具查找", std::env::consts::OS),
            ))
        }
    }

    /// 解析 macOS 应用包中的 Marker 工具路径
    /// 
    /// macOS 应用包结构：
    /// AppFlowy.app/
    /// └── Contents/
    ///     ├── MacOS/AppFlowy (可执行文件)
    ///     └── Resources/
    ///         └── marker/
    ///             └── marker (Marker 工具)
    fn resolve_macos_bundle_path(&self, exe_path: &Path) -> FlowyResult<PathBuf> {
        // 检查是否在 .app 包中
        // 如果在 .app 包中，exe_path 应该是 AppFlowy.app/Contents/MacOS/AppFlowy
        // 我们需要找到 AppFlowy.app/Contents/Resources/marker/marker
        
        let mut current = exe_path.to_path_buf();
        
        // 向上查找 .app 包
        // 从 MacOS 目录向上到 Contents，再到 .app 包
        while let Some(parent) = current.parent() {
            // 检查是否是 Contents 目录
            if parent.file_name().and_then(|n| n.to_str()) == Some("Contents") {
                // 检查父目录是否是 .app 包
                if let Some(grandparent) = parent.parent() {
                    if grandparent.file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.ends_with(".app"))
                        .unwrap_or(false)
                    {
                        // 构建 Resources/marker/marker 路径
                        let marker_path = parent.join("Resources").join("marker").join("marker");
                        debug!("构建的 macOS Marker 路径: {}", marker_path.display());
                        return Ok(marker_path);
                    }
                }
            }
            
            // 检查当前路径是否是 .app 包
            if current.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.ends_with(".app"))
                .unwrap_or(false)
            {
                // 构建 Contents/Resources/marker/marker 路径
                let marker_path = current.join("Contents").join("Resources").join("marker").join("marker");
                debug!("构建的 macOS Marker 路径: {}", marker_path.display());
                return Ok(marker_path);
            }
            
            current = parent.to_path_buf();
            
            // 防止无限循环
            if current == PathBuf::from("/") {
                break;
            }
        }
        
        // 如果无法从 .app 包结构推断，尝试从可执行文件路径直接构建
        // 假设可执行文件在 MacOS 目录下
        if let Some(parent) = exe_path.parent() {
            if parent.file_name().and_then(|n| n.to_str()) == Some("MacOS") {
                if let Some(contents_dir) = parent.parent() {
                    let marker_path = contents_dir.join("Resources").join("marker").join("marker");
                    debug!("从 MacOS 目录推断的 Marker 路径: {}", marker_path.display());
                    return Ok(marker_path);
                }
            }
        }
        
        Err(FlowyError::new(
            flowy_error::ErrorCode::Internal,
            format!(
                "无法从可执行文件路径推断 macOS 应用包路径: {}\n\
                期望的应用包结构: AppFlowy.app/Contents/Resources/marker/marker",
                exe_path.display()
            ),
        ))
    }

    /// 解析 Windows 应用包中的 Marker 工具路径
    /// 
    /// Windows 应用目录结构：
    /// AppFlowy/
    /// ├── AppFlowy.exe (可执行文件)
    /// └── Resources/
    ///     └── marker/
    ///         └── marker.exe (Marker 工具)
    fn resolve_windows_bundle_path(&self, exe_path: &Path) -> FlowyResult<PathBuf> {
        // 获取可执行文件所在目录
        let app_dir = exe_path.parent().ok_or_else(|| {
            FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("无法获取可执行文件的父目录: {}", exe_path.display()),
            )
        })?;

        // 构建 Resources/marker/marker.exe 路径
        let marker_path = app_dir.join("Resources").join("marker").join("marker.exe");
        debug!("构建的 Windows Marker 路径: {}", marker_path.display());
        
        Ok(marker_path)
    }

    /// 验证 Marker 工具是否可用
    /// 
    /// 检查：
    /// 1. 文件是否存在
    /// 2. 文件是否可执行（在 Unix 系统上检查执行权限）
    /// 
    /// 如果验证失败，返回详细的错误信息。
    pub fn verify_marker(&self, path: &Path) -> FlowyResult<()> {
        // 检查文件是否存在
        if !path.exists() {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::RecordNotFound,
                format!(
                    "Marker 工具未找到: {}\n\
                    请确保 Marker 工具已正确打包到应用包中。\n\
                    macOS 期望路径: AppFlowy.app/Contents/Resources/marker/marker\n\
                    Windows 期望路径: AppFlowy/Resources/marker/marker.exe",
                    path.display()
                ),
            ));
        }

        // 检查是否是文件（而不是目录）
        if !path.is_file() {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::InvalidParams,
                format!(
                    "Marker 工具路径指向的不是文件: {}\n\
                    请检查应用包中的 Marker 工具是否正确安装。",
                    path.display()
                ),
            ));
        }

        // 在 Unix 系统上检查执行权限
        #[cfg(unix)]
        {
            use std::fs::Permissions;
            use std::os::unix::fs::PermissionsExt;
            
            let metadata = path.metadata().map_err(|e| {
                FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("无法获取文件元数据 {}: {}", path.display(), e),
                )
            })?;
            
            let permissions = metadata.permissions();
            let mode = permissions.mode();
            
            // 检查是否有执行权限（用户、组或其他）
            if mode & 0o111 == 0 {
                warn!(
                    "Marker 工具可能没有执行权限: {} (权限: {:o})",
                    path.display(),
                    mode
                );
                // 尝试添加执行权限
                std::fs::set_permissions(path, Permissions::from_mode(mode | 0o111)).map_err(|e| {
                    FlowyError::new(
                        flowy_error::ErrorCode::Internal,
                        format!(
                            "Marker 工具没有执行权限，且无法添加执行权限: {}: {}",
                            path.display(),
                            e
                        ),
                    )
                })?;
                info!("已为 Marker 工具添加执行权限: {}", path.display());
            }
        }

        debug!("Marker 工具验证成功: {}", path.display());
        Ok(())
    }

    /// 获取 Marker 工具路径
    /// 
    /// 如果路径已缓存，直接返回缓存的路径。
    /// 如果未缓存，则从应用包中查找并验证，然后缓存结果。
    /// 
    /// 返回 Marker 工具的路径，如果未找到则返回错误。
    pub fn get_marker_path(&self) -> FlowyResult<PathBuf> {
        // 检查是否已缓存
        {
            let marker_path = self.marker_path.lock().unwrap();
            if let Some(ref path) = *marker_path {
                return Ok(path.clone());
            }
        }

        // 检查是否已搜索过
        {
            let searched = self.searched.lock().unwrap();
            if *searched {
                // 已搜索过但未找到，返回错误
                return Err(FlowyError::new(
                    flowy_error::ErrorCode::RecordNotFound,
                    "Marker 工具未找到。应用包中可能缺少 Marker 工具，请重新安装应用。".to_string(),
                ));
            }
        }

        // 首次查找
        debug!("首次查找 Marker 工具...");
        let marker_path = match self.find_marker_in_bundle() {
            Ok(path) => {
                // 缓存路径
                let mut cached_path = self.marker_path.lock().unwrap();
                *cached_path = Some(path.clone());
                path
            }
            Err(e) => {
                // 记录详细的错误信息
                warn!("Marker 工具查找失败: {}", e);
                debug!("Marker 工具查找失败详情: {:?}", e);
                // 标记为已搜索
                let mut searched = self.searched.lock().unwrap();
                *searched = true;
                return Err(e);
            }
        };

        // 标记为已搜索
        {
            let mut searched = self.searched.lock().unwrap();
            *searched = true;
        }

        Ok(marker_path)
    }

    /// 检查 Marker 工具是否可用
    /// 
    /// 如果工具已缓存且可用，返回 true；否则尝试查找并验证。
    /// 如果查找失败，返回 false（不会返回错误，只返回布尔值）。
    pub fn is_available(&self) -> bool {
        self.get_marker_path().is_ok()
    }

    /// 清除缓存的路径（用于测试或重新查找）
    pub fn clear_cache(&self) {
        let mut marker_path = self.marker_path.lock().unwrap();
        *marker_path = None;
        let mut searched = self.searched.lock().unwrap();
        *searched = false;
    }
}

impl Default for MarkerToolManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_marker_tool_manager_creation() {
        let manager = MarkerToolManager::new();
        assert!(!manager.is_available()); // 在没有实际工具的情况下应该返回 false
    }

    #[test]
    fn test_marker_tool_manager_default() {
        let manager = MarkerToolManager::default();
        assert_eq!(manager.get_marker_path().is_err(), true);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_resolve_macos_bundle_path() {
        let manager = MarkerToolManager::new();
        
        // 创建临时 .app 包结构
        let temp_dir = TempDir::new().unwrap();
        let app_dir = temp_dir.path().join("AppFlowy.app");
        let contents_dir = app_dir.join("Contents");
        let macos_dir = contents_dir.join("MacOS");
        let resources_dir = contents_dir.join("Resources");
        let marker_dir = resources_dir.join("marker");
        
        fs::create_dir_all(&macos_dir).unwrap();
        fs::create_dir_all(&marker_dir).unwrap();
        
        let exe_path = macos_dir.join("AppFlowy");
        
        // 创建虚拟可执行文件
        fs::File::create(&exe_path).unwrap();
        
        let result = manager.resolve_macos_bundle_path(&exe_path);
        assert!(result.is_ok());
        let marker_path = result.unwrap();
        assert_eq!(marker_path, marker_dir.join("marker"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_resolve_windows_bundle_path() {
        let manager = MarkerToolManager::new();
        
        // 创建临时应用目录结构
        let temp_dir = TempDir::new().unwrap();
        let app_dir = temp_dir.path();
        let resources_dir = app_dir.join("Resources");
        let marker_dir = resources_dir.join("marker");
        
        fs::create_dir_all(&marker_dir).unwrap();
        
        let exe_path = app_dir.join("AppFlowy.exe");
        
        // 创建虚拟可执行文件
        fs::File::create(&exe_path).unwrap();
        
        let result = manager.resolve_windows_bundle_path(&exe_path);
        assert!(result.is_ok());
        let marker_path = result.unwrap();
        assert_eq!(marker_path, marker_dir.join("marker.exe"));
    }

    #[test]
    fn test_verify_marker_nonexistent() {
        let manager = MarkerToolManager::new();
        let temp_dir = TempDir::new().unwrap();
        let fake_path = temp_dir.path().join("nonexistent");
        
        let result = manager.verify_marker(&fake_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_cache() {
        let manager = MarkerToolManager::new();
        manager.clear_cache();
        // 应该能够清除缓存（不会出错）
        assert!(!manager.is_available());
    }
}

