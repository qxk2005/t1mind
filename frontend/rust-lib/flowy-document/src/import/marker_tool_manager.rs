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
    ///         └── marker.exe 或 marker.bat (Marker 工具)
    fn resolve_windows_bundle_path(&self, exe_path: &Path) -> FlowyResult<PathBuf> {
        // 获取可执行文件所在目录
        let app_dir = exe_path.parent().ok_or_else(|| {
            FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("无法获取可执行文件的父目录: {}", exe_path.display()),
            )
        })?;

        // 首先尝试 marker.exe
        let marker_exe_path = app_dir.join("Resources").join("marker").join("marker.exe");
        if marker_exe_path.exists() {
            debug!("构建的 Windows Marker 路径: {}", marker_exe_path.display());
            return Ok(marker_exe_path);
        }
        
        // 如果 marker.exe 不存在，尝试 marker.bat
        let marker_bat_path = app_dir.join("Resources").join("marker").join("marker.bat");
        if marker_bat_path.exists() {
            debug!("构建的 Windows Marker 路径: {}", marker_bat_path.display());
            return Ok(marker_bat_path);
        }
        
        // 如果都不存在，返回 marker.exe 路径（用于错误提示）
        debug!("构建的 Windows Marker 路径: {}", marker_exe_path.display());
        Ok(marker_exe_path)
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
        
        // 可选：检查 marker-pdf 依赖是否可用（仅警告，不阻止使用）
        // 这可以帮助提前发现问题，但不会阻止工具的使用
        // 因为 marker 脚本本身会在运行时检查依赖
        #[cfg(unix)]
        {
            if let Err(e) = self.check_marker_pdf_dependency() {
                warn!(
                    "Marker 工具依赖检查失败: {}\n\
                    这不会阻止 Marker 工具的使用，但如果 marker-pdf 未安装，\n\
                    在使用时会失败并显示安装说明。",
                    e
                );
            }
        }
        
        Ok(())
    }
    
    /// 检查 marker-pdf 依赖是否可用
    /// 
    /// 这是一个可选的检查，用于提前发现问题。
    /// 如果检查失败，只记录警告，不会阻止工具的使用。
    fn check_marker_pdf_dependency(&self) -> Result<(), String> {
        use std::process::Command;
        
        // 根据平台检查 marker-pdf 是否安装
        #[cfg(target_os = "macos")]
        {
            // 检测是否安装了 Homebrew
            // 首先尝试使用 which 检查（如果 brew 在 PATH 中）
            let brew_available_from_path = Command::new("which")
                .arg("brew")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false);
            
            // 如果 which 找不到，检查默认安装路径
            let arch = Command::new("uname")
                .arg("-m")
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .unwrap_or_else(|| "x86_64".to_string())
                .trim()
                .to_string();
            let is_apple_silicon = arch == "arm64";
            
            let brew_path = if is_apple_silicon {
                "/opt/homebrew/bin/brew"
            } else {
                "/usr/local/bin/brew"
            };
            
            let brew_available_from_path_check = std::path::Path::new(brew_path).exists();
            
            // 如果路径存在，尝试执行 brew --version 来验证
            let brew_available = if brew_available_from_path {
                true
            } else if brew_available_from_path_check {
                // 检查文件是否存在且可执行
                Command::new(brew_path)
                    .arg("--version")
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            } else {
                false
            };
            
            if !brew_available {
                return Err(format!(
                    "Homebrew 未安装。\n\n\
                    Marker 工具需要使用 Homebrew 来安装 marker-pdf。\n\n\
                    请先安装 Homebrew：\n\
                    1. 打开终端，运行：\n\
                       /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"\n\n\
                    2. 或者访问 https://brew.sh 查看安装说明\n\n\
                    3. 安装 Homebrew 后，运行：\n\
                       brew install jpeg libpng freetype openjpeg libtiff webp\n\
                       brew install pipx\n\
                       pipx install marker-pdf"
                ));
            }
            
            // 检查 pipx 是否可用
            let pipx_check = Command::new("which")
                .arg("pipx")
                .output();
            
            let pipx_available = match pipx_check {
                Ok(output) => output.status.success(),
                Err(_) => false,
            };
            
            // 检查 marker-pdf 是否已安装
            let home_dir = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
            let marker_pdf_path = std::path::Path::new(&home_dir)
                .join(".local")
                .join("pipx")
                .join("venvs")
                .join("marker-pdf")
                .join("bin")
                .join("marker_single");
            
            if !marker_pdf_path.exists() {
                return Err(format!(
                    "marker-pdf 未安装。\n\n\
                    安装方法（使用 Homebrew）：\n\
                    1. 首先安装 Pillow 编译所需的依赖库：\n\
                       brew install jpeg libpng freetype openjpeg libtiff webp\n\
                    2. 安装 pipx：\n\
                       brew install pipx\n\
                    3. 安装 marker-pdf：\n\
                       pipx install marker-pdf\n\n\
                    期望位置: {}",
                    marker_pdf_path.display()
                ));
            }
            
            if !pipx_available {
                return Err(format!(
                    "pipx 未安装。\n\n\
                    安装方法（使用 Homebrew）：\n\
                    1. 首先安装 Pillow 编译所需的依赖库：\n\
                       brew install jpeg libpng freetype openjpeg libtiff webp\n\
                    2. 安装 pipx：\n\
                       brew install pipx\n\
                    3. 安装 marker-pdf：\n\
                       pipx install marker-pdf"
                ));
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            // Windows 下检查 pipx 是否可用
            let pipx_available = Command::new("where")
                .arg("pipx")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false);
            
            // 检查 Python 是否可用
            let python_available = Command::new("where")
                .arg("python")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false);
            
            if !python_available {
                return Err(format!(
                    "Python 未安装。\n\n\
                    首先需要安装 Python 3.8+：\n\n\
                    方法 1：使用 WinGet 安装（推荐，Windows 10/11 自带）\n\
                      winget install Python.Python.3.12\n\n\
                    方法 2：手动安装\n\
                    1. 访问 https://www.python.org/downloads/ 下载并安装 Python\n\
                    2. 安装时勾选 \"Add Python to PATH\"\n\n\
                    安装 Python 后，再安装 marker-pdf：\n\
                    1. 打开命令提示符或 PowerShell\n\
                    2. 运行: pip install --user pipx\n\
                    3. 运行: pipx install marker-pdf"
                ));
            }
            
            // 检查 marker-pdf 是否已安装
            let localappdata = std::env::var("LOCALAPPDATA")
                .or_else(|_| std::env::var("USERPROFILE").map(|p| format!("{}\\AppData\\Local", p)))
                .unwrap_or_else(|_| "%LOCALAPPDATA%".to_string());
            
            let marker_pdf_path1 = std::path::Path::new(&localappdata)
                .join("pipx")
                .join("venvs")
                .join("marker-pdf")
                .join("Scripts")
                .join("marker_single.exe");
            
            let marker_pdf_path2 = if let Ok(userprofile) = std::env::var("USERPROFILE") {
                Some(std::path::Path::new(&userprofile)
                    .join(".local")
                    .join("pipx")
                    .join("venvs")
                    .join("marker-pdf")
                    .join("Scripts")
                    .join("marker_single.exe"))
            } else {
                None
            };
            
            let marker_pdf_exists = marker_pdf_path1.exists() 
                || marker_pdf_path2.as_ref().map(|p| p.exists()).unwrap_or(false);
            
            if !marker_pdf_exists {
                if !pipx_available {
                    return Err(format!(
                        "marker-pdf 未安装。\n\n\
                        pipx 未安装。请先安装 pipx：\n\n\
                        方法 1：使用 WinGet 安装（推荐，Windows 10/11 自带）\n\
                        注意：WinGet 会自动安装 Python 作为 pipx 的依赖\n\
                          winget install pipx\n\n\
                        方法 2：使用 pip 安装（需要先安装 Python）\n\
                        1. 打开命令提示符或 PowerShell\n\
                        2. 运行: pip install --user pipx\n\
                        3. 将 pipx 添加到 PATH（如果尚未添加）\n\n\
                        安装 pipx 后，运行: pipx install marker-pdf"
                    ));
                }
                
                return Err(format!(
                    "marker-pdf 未安装。\n\n\
                    安装方法：\n\
                    1. 打开命令提示符或 PowerShell（以管理员身份运行）\n\
                    2. 运行: pipx install marker-pdf\n\n\
                    注意：Windows 下 Pillow 使用预编译包，通常不需要手动安装依赖库。\n\n\
                    期望位置: {} 或 {}",
                    marker_pdf_path1.display(),
                    marker_pdf_path2.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "%USERPROFILE%\\.local\\pipx\\venvs\\marker-pdf\\Scripts\\marker_single.exe".to_string())
                ));
            }
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            return Err(format!("当前平台 {} 不支持 marker-pdf 检查", std::env::consts::OS));
        }
        
        debug!("Marker-pdf 依赖检查通过");
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

