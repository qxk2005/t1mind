use std::sync::Arc;
use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::OnceLock;

/// 日志回调函数类型
pub type LogCallback = Box<dyn Fn(&str, &str, String) + Send + Sync>;
/// 进度回调函数类型
pub type ProgressCallback = Box<dyn Fn(&str, &str, f64, &str) + Send + Sync>;

/// 全局的导入日志收集器
/// 用于在 MarkerPdfConverter 中收集日志并发送到 FolderManager
static IMPORT_LOG_COLLECTORS: OnceLock<RwLock<HashMap<String, (Arc<LogCallback>, Arc<ProgressCallback>)>>> = OnceLock::new();

fn get_collectors() -> &'static RwLock<HashMap<String, (Arc<LogCallback>, Arc<ProgressCallback>)>> {
    IMPORT_LOG_COLLECTORS.get_or_init(|| RwLock::new(HashMap::new()))
}

/// 注册导入日志收集器
pub fn register_import_log_collector(
    import_id: String,
    log_callback: Arc<LogCallback>,
    progress_callback: Arc<ProgressCallback>,
) {
    if let Ok(mut collectors) = get_collectors().write() {
        collectors.insert(import_id, (log_callback, progress_callback));
    }
}

/// 取消注册导入日志收集器
pub fn unregister_import_log_collector(import_id: &str) {
    if let Ok(mut collectors) = get_collectors().write() {
        collectors.remove(import_id);
    }
}

/// 发送导入日志
pub fn send_import_log(import_id: &str, level: &str, message: String) {
    if let Ok(collectors) = get_collectors().read() {
        if let Some((log_callback, _)) = collectors.get(import_id) {
            log_callback(import_id, level, message);
        }
    }
}

/// 发送导入进度
pub fn send_import_progress(import_id: &str, file_name: &str, progress: f64, step: &str) {
    if let Ok(collectors) = get_collectors().read() {
        if let Some((_, progress_callback)) = collectors.get(import_id) {
            progress_callback(import_id, file_name, progress, step);
        }
    }
}

