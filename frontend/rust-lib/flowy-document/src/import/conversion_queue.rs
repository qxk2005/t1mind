use crate::import::{ConversionTask, ConversionStatus, ConversionResult, ConverterFactory, DefaultConverterFactory};
use flowy_error::{FlowyError, FlowyResult};
use lib_infra::priority_task::{TaskDispatcher, TaskHandler, TaskContent};
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use std::path::Path;
use std::sync::atomic::{AtomicU8, AtomicBool};
use std::sync::{Arc, Weak};
use std::time::Duration;
use tokio::sync::{RwLock, watch, mpsc};
use tokio::time::interval;
use tracing::{error, info, trace};
use uuid::Uuid;

/// 转换任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConversionPriority {
    /// 低优先级
    Low = 1,
    /// 普通优先级
    Normal = 2,
    /// 高优先级
    High = 3,
    /// 紧急优先级
    Urgent = 4,
}

/// 转换任务队列项
#[derive(Debug, Clone)]
pub struct ConversionQueueItem {
    /// 任务ID
    pub task_id: Uuid,
    /// 优先级
    pub priority: ConversionPriority,
    /// 创建时间戳
    pub created_at: u64,
    /// 重试次数
    pub retry_count: u8,
}

impl ConversionQueueItem {
    pub fn new(task_id: Uuid, priority: ConversionPriority) -> Self {
        Self {
            task_id,
            priority,
            created_at: chrono::Utc::now().timestamp() as u64,
            retry_count: 0,
        }
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

impl PartialEq for ConversionQueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.task_id == other.task_id
    }
}

impl Eq for ConversionQueueItem {}

impl PartialOrd for ConversionQueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ConversionQueueItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap是最大堆，所以高优先级（值更大）的会先被处理
        // 首先按优先级排序（高优先级在前）
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                // 优先级相同时，按创建时间排序（早创建的在前）
                other.created_at.cmp(&self.created_at)
            }
            other => other,
        }
    }
}

/// 转换任务队列
pub struct ConversionTaskQueue {
    /// 任务队列（使用BinaryHeap实现优先级队列）
    queue: RwLock<BinaryHeap<ConversionQueueItem>>,
    /// 任务存储
    tasks: RwLock<HashMap<Uuid, ConversionTask>>,
    /// 进度通知通道
    progress_sender: mpsc::UnboundedSender<ProgressUpdate>,
    /// 最大并发任务数
    max_concurrent_tasks: AtomicU8,
    /// 当前运行的任务数
    current_tasks: AtomicU8,
    /// 队列是否暂停
    paused: AtomicBool,
    /// 转换器工厂
    converter_factory: Arc<dyn ConverterFactory>,
}

/// 进度更新消息
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub task_id: Uuid,
    pub status: ConversionStatus,
    pub progress: u8,
    pub error_message: Option<String>,
}

/// 转换任务队列管理器
pub struct ConversionQueueManager {
    queue: Arc<ConversionTaskQueue>,
    dispatcher: Arc<RwLock<TaskDispatcher>>,
    notifier: watch::Sender<bool>,
    progress_receiver: Option<mpsc::UnboundedReceiver<ProgressUpdate>>,
}

impl ConversionTaskQueue {
    pub fn new(
        max_concurrent_tasks: u8,
        converter_factory: Arc<dyn ConverterFactory>,
    ) -> (Self, mpsc::UnboundedReceiver<ProgressUpdate>) {
        let (progress_sender, progress_receiver) = mpsc::unbounded_channel();
        
        let queue = Self {
            queue: RwLock::new(BinaryHeap::new()),
            tasks: RwLock::new(HashMap::new()),
            progress_sender,
            max_concurrent_tasks: AtomicU8::new(max_concurrent_tasks),
            current_tasks: AtomicU8::new(0),
            paused: AtomicBool::new(false),
            converter_factory,
        };

        (queue, progress_receiver)
    }

    /// 添加转换任务到队列
    pub async fn add_task(&self, task: ConversionTask, priority: ConversionPriority) -> FlowyResult<()> {
        let task_id = task.id;
        
        // 存储任务
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id, task);
        }

        // 添加到队列
        {
            let mut queue = self.queue.write().await;
            let queue_item = ConversionQueueItem::new(task_id, priority);
            queue.push(queue_item);
        }

        // 发送进度更新
        let _ = self.progress_sender.send(ProgressUpdate {
            task_id,
            status: ConversionStatus::Pending,
            progress: 0,
            error_message: None,
        });

        trace!("Added conversion task {} to queue with priority {:?}", task_id, priority);
        Ok(())
    }

    /// 获取下一个待处理的任务
    pub async fn get_next_task(&self) -> Option<ConversionTask> {
        // 检查是否暂停
        if self.paused.load(std::sync::atomic::Ordering::Relaxed) {
            return None;
        }

        // 检查并发限制
        let current_tasks = self.current_tasks.load(std::sync::atomic::Ordering::SeqCst);
        let max_tasks = self.max_concurrent_tasks.load(std::sync::atomic::Ordering::SeqCst);
        
        if current_tasks >= max_tasks {
            trace!("Max concurrent tasks reached: {}/{}", current_tasks, max_tasks);
            return None;
        }

        // 从队列中获取下一个任务
        let queue_item = {
            let mut queue = self.queue.write().await;
            queue.pop()
        }?;

        // 获取任务详情
        let mut tasks = self.tasks.write().await;
        let task = tasks.get_mut(&queue_item.task_id)?;

        // 检查任务状态
        if task.status != ConversionStatus::Pending {
            return None;
        }

        // 标记任务为处理中
        task.mark_processing();
        
        // 增加当前任务计数
        self.current_tasks.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // 发送进度更新
        let _ = self.progress_sender.send(ProgressUpdate {
            task_id: task.id,
            status: ConversionStatus::Processing,
            progress: 0,
            error_message: None,
        });

        Some(task.clone())
    }

    /// 完成任务
    pub async fn complete_task(&self, task_id: Uuid, result: FlowyResult<ConversionResult>) {
        let mut tasks = self.tasks.write().await;
        
        if let Some(task) = tasks.get_mut(&task_id) {
            match result {
                Ok(_) => {
                    task.mark_completed();
                    info!("Conversion task {} completed successfully", task_id);
                }
                Err(err) => {
                    task.mark_failed(err.to_string());
                    error!("Conversion task {} failed: {}", task_id, err);
                }
            }

            // 发送进度更新
            let _ = self.progress_sender.send(ProgressUpdate {
                task_id,
                status: task.status.clone(),
                progress: task.progress,
                error_message: task.error_message.clone(),
            });
        }

        // 减少当前任务计数
        self.current_tasks.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: Uuid) -> FlowyResult<()> {
        let mut tasks = self.tasks.write().await;
        
        if let Some(task) = tasks.get_mut(&task_id) {
            if task.status == ConversionStatus::Processing {
                // 如果任务正在处理，需要停止处理
                task.cancel();
                
                // 减少当前任务计数
                self.current_tasks.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            } else if task.status == ConversionStatus::Pending {
                // 如果任务还在队列中，直接取消
                task.cancel();
                
                // 从队列中移除
                let mut queue = self.queue.write().await;
                queue.retain(|item| item.task_id != task_id);
            }

            // 发送进度更新
            let _ = self.progress_sender.send(ProgressUpdate {
                task_id,
                status: ConversionStatus::Cancelled,
                progress: task.progress,
                error_message: None,
            });

            Ok(())
        } else {
            Err(FlowyError::new(flowy_error::ErrorCode::RecordNotFound, "Task not found"))
        }
    }

    /// 获取任务状态
    pub async fn get_task_status(&self, task_id: Uuid) -> Option<ConversionStatus> {
        let tasks = self.tasks.read().await;
        tasks.get(&task_id).map(|task| task.status.clone())
    }

    /// 获取所有任务
    pub async fn get_all_tasks(&self) -> Vec<ConversionTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// 获取队列统计信息
    pub async fn get_queue_stats(&self) -> QueueStats {
        let tasks = self.tasks.read().await;
        let queue = self.queue.read().await;
        
        let mut stats = QueueStats::default();
        
        for task in tasks.values() {
            match task.status {
                ConversionStatus::Pending => stats.pending += 1,
                ConversionStatus::Processing => stats.processing += 1,
                ConversionStatus::Completed => stats.completed += 1,
                ConversionStatus::Failed => stats.failed += 1,
                ConversionStatus::Cancelled => stats.cancelled += 1,
            }
        }
        
        stats.queue_size = queue.len() as u32;
        stats.current_tasks = self.current_tasks.load(std::sync::atomic::Ordering::SeqCst);
        stats.max_concurrent_tasks = self.max_concurrent_tasks.load(std::sync::atomic::Ordering::SeqCst);
        
        stats
    }

    /// 暂停队列
    pub fn pause(&self) {
        self.paused.store(true, std::sync::atomic::Ordering::SeqCst);
        info!("Conversion queue paused");
    }

    /// 恢复队列
    pub fn resume(&self) {
        self.paused.store(false, std::sync::atomic::Ordering::SeqCst);
        info!("Conversion queue resumed");
    }

    /// 设置最大并发任务数
    pub fn set_max_concurrent_tasks(&self, max_tasks: u8) {
        self.max_concurrent_tasks.store(max_tasks, std::sync::atomic::Ordering::SeqCst);
        info!("Max concurrent tasks set to {}", max_tasks);
    }

    /// 清理已完成的任务
    pub async fn cleanup_completed_tasks(&self, older_than_hours: u64) {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::hours(older_than_hours as i64);
        
        let mut tasks = self.tasks.write().await;
        let mut queue = self.queue.write().await;
        
        let mut to_remove = Vec::new();
        
        for (task_id, task) in tasks.iter() {
            if matches!(task.status, ConversionStatus::Completed | ConversionStatus::Failed | ConversionStatus::Cancelled) {
                if let Some(completed_at) = task.completed_at {
                    if completed_at < cutoff_time {
                        to_remove.push(*task_id);
                    }
                }
            }
        }
        
        let cleanup_count = to_remove.len();
        for task_id in to_remove {
            tasks.remove(&task_id);
            queue.retain(|item| item.task_id != task_id);
        }
        
        info!("Cleaned up {} completed tasks", cleanup_count);
    }
}

/// 队列统计信息
#[derive(Debug, Default)]
pub struct QueueStats {
    pub pending: u32,
    pub processing: u32,
    pub completed: u32,
    pub failed: u32,
    pub cancelled: u32,
    pub queue_size: u32,
    pub current_tasks: u8,
    pub max_concurrent_tasks: u8,
}

impl ConversionQueueManager {
    pub fn new(max_concurrent_tasks: u8) -> Self {
        let converter_factory = Arc::new(DefaultConverterFactory);
        let (queue, progress_receiver) = ConversionTaskQueue::new(max_concurrent_tasks, converter_factory);
        
        let (notifier, _) = watch::channel(false);
        let dispatcher = Arc::new(RwLock::new(TaskDispatcher::new(Duration::from_secs(300)))); // 5分钟超时
        
        Self {
            queue: Arc::new(queue),
            dispatcher,
            notifier,
            progress_receiver: Some(progress_receiver),
        }
    }

    /// 添加转换任务
    pub async fn add_conversion_task(
        &self,
        source_path: String,
        document_type: crate::import::DocumentType,
        target_name: String,
        priority: ConversionPriority,
    ) -> FlowyResult<Uuid> {
        // 验证文件
        let path = Path::new(&source_path);
        if !path.exists() {
            return Err(FlowyError::new(flowy_error::ErrorCode::RecordNotFound, "File not found"));
        }

        // 创建转换任务
        let task = ConversionTask::new(
            source_path,
            document_type,
            target_name,
            crate::import::ConversionConfig::default(),
        );

        let task_id = task.id;

        // 添加到队列
        self.queue.add_task(task, priority).await?;

        // 通知调度器
        self.notify_scheduler().await;

        Ok(task_id)
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: Uuid) -> FlowyResult<()> {
        self.queue.cancel_task(task_id).await
    }

    /// 获取任务状态
    pub async fn get_task_status(&self, task_id: Uuid) -> Option<ConversionStatus> {
        self.queue.get_task_status(task_id).await
    }

    /// 获取所有任务
    pub async fn get_all_tasks(&self) -> Vec<ConversionTask> {
        self.queue.get_all_tasks().await
    }

    /// 获取队列统计信息
    pub async fn get_queue_stats(&self) -> QueueStats {
        self.queue.get_queue_stats().await
    }

    /// 暂停队列
    pub fn pause(&self) {
        self.queue.pause();
    }

    /// 恢复队列
    pub fn resume(&self) {
        self.queue.resume();
    }

    /// 设置最大并发任务数
    pub fn set_max_concurrent_tasks(&self, max_tasks: u8) {
        self.queue.set_max_concurrent_tasks(max_tasks);
    }

    /// 清理已完成的任务
    pub async fn cleanup_completed_tasks(&self, older_than_hours: u64) {
        self.queue.cleanup_completed_tasks(older_than_hours).await;
    }

    /// 获取进度接收器
    pub fn take_progress_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<ProgressUpdate>> {
        self.progress_receiver.take()
    }

    /// 启动队列处理器
    pub async fn start(&self) {
        let queue = Arc::downgrade(&self.queue);
        let notifier = self.notifier.subscribe();
        
        tokio::spawn(async move {
            Self::run_queue_processor(queue, notifier).await;
        });
    }

    /// 停止队列处理器
    pub fn stop(&self) {
        let _ = self.notifier.send(true);
    }

    /// 通知调度器
    async fn notify_scheduler(&self) {
        let _ = self.notifier.send(false);
    }

    /// 队列处理器主循环
    async fn run_queue_processor(
        queue: Weak<ConversionTaskQueue>,
        mut notifier: watch::Receiver<bool>,
    ) {
        let mut interval = interval(Duration::from_millis(100));
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Some(queue) = queue.upgrade() {
                        Self::process_next_conversion_task(queue).await;
                    } else {
                        info!("Conversion queue dropped, stopping processor");
                        break;
                    }
                }
                _ = notifier.changed() => {
                    if *notifier.borrow() {
                        info!("Conversion queue processor stopped by signal");
                        break;
                    }
                }
            }
        }
    }

    /// 处理下一个转换任务
    async fn process_next_conversion_task(queue: Arc<ConversionTaskQueue>) {
        if let Some(task) = queue.get_next_task().await {
            let task_id = task.id;
            let document_type = task.document_type.clone();
            let converter_factory = queue.converter_factory.clone();
            
            // 在后台处理任务
            let queue_clone = Arc::clone(&queue);
            tokio::spawn(async move {
                let result = Self::execute_conversion_task(task, document_type, converter_factory).await;
                queue_clone.complete_task(task_id, result).await;
            });
        }
    }

    /// 执行转换任务
    async fn execute_conversion_task(
        task: ConversionTask,
        document_type: crate::import::DocumentType,
        converter_factory: Arc<dyn ConverterFactory>,
    ) -> FlowyResult<ConversionResult> {
        // 创建转换器
        let converter = converter_factory
            .create_converter(document_type)
            .ok_or_else(|| FlowyError::new(flowy_error::ErrorCode::UnsupportedFileFormat, "Unsupported document type"))?;

        // 执行转换
        converter.convert(&task).await
    }
}

/// 转换任务处理器，用于集成TaskScheduler
pub struct ConversionTaskHandler {
    queue_manager: Arc<ConversionQueueManager>,
}

impl ConversionTaskHandler {
    pub fn new(queue_manager: Arc<ConversionQueueManager>) -> Self {
        Self { queue_manager }
    }
}

#[async_trait::async_trait]
impl TaskHandler for ConversionTaskHandler {
    fn handler_id(&self) -> &str {
        "conversion_task_handler"
    }

    fn handler_name(&self) -> &str {
        "Document Conversion Task Handler"
    }

    async fn run(&self, _content: TaskContent) -> Result<(), anyhow::Error> {
        // 这里可以处理来自TaskScheduler的任务
        // 目前主要使用内部的队列处理器
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::{DocumentType, ConversionConfig};

    #[tokio::test]
    async fn test_conversion_queue_basic_operations() {
        let converter_factory = Arc::new(DefaultConverterFactory);
        let (queue, mut progress_receiver) = ConversionTaskQueue::new(2, converter_factory);

        // 创建测试任务
        let task = ConversionTask::new(
            "test.docx".to_string(),
            DocumentType::Word,
            "test".to_string(),
            ConversionConfig::default(),
        );

        let task_id = task.id;

        // 添加任务
        queue.add_task(task, ConversionPriority::Normal).await.unwrap();

        // 检查任务状态
        let status = queue.get_task_status(task_id).await.unwrap();
        assert_eq!(status, ConversionStatus::Pending);

        // 获取下一个任务
        let next_task = queue.get_next_task().await.unwrap();
        assert_eq!(next_task.id, task_id);
        assert_eq!(next_task.status, ConversionStatus::Processing);

        // 完成任务
        let result = ConversionResult {
            task_id,
            document_content: crate::import::DocumentContent {
                title: "Test".to_string(),
                content: "Test content".to_string(),
                metadata: crate::import::DocumentMetadata {
                    author: None,
                    created_at: None,
                    modified_at: None,
                    page_count: None,
                    word_count: None,
                    language: None,
                },
            },
            extracted_images: vec![],
            statistics: crate::import::ConversionStatistics {
                processing_time_ms: 1000,
                text_length: 12,
                image_count: 0,
                table_count: 0,
                source_file_size: 1024,
            },
        };

        queue.complete_task(task_id, Ok(result)).await;

        // 检查最终状态
        let final_status = queue.get_task_status(task_id).await.unwrap();
        assert_eq!(final_status, ConversionStatus::Completed);
    }

    #[tokio::test]
    async fn test_conversion_queue_priority() {
        let converter_factory = Arc::new(DefaultConverterFactory);
        let (queue, _progress_receiver) = ConversionTaskQueue::new(2, converter_factory);

        // 创建不同优先级的任务
        let low_task = ConversionTask::new(
            "low.docx".to_string(),
            DocumentType::Word,
            "low".to_string(),
            ConversionConfig::default(),
        );

        let high_task = ConversionTask::new(
            "high.docx".to_string(),
            DocumentType::Word,
            "high".to_string(),
            ConversionConfig::default(),
        );

        // 先添加低优先级任务
        queue.add_task(low_task.clone(), ConversionPriority::Low).await.unwrap();
        queue.add_task(high_task.clone(), ConversionPriority::High).await.unwrap();

        // 高优先级任务应该先被处理
        let next_task = queue.get_next_task().await.unwrap();
        // 由于BinaryHeap是最大堆，高优先级（值更大）的任务会先被处理
        assert_eq!(next_task.id, high_task.id);
    }

    #[tokio::test]
    async fn test_conversion_queue_manager() {
        let manager = ConversionQueueManager::new(2);

        // 创建一个临时文件用于测试
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test.docx");
        std::fs::write(&test_file, b"test content").unwrap();

        // 添加任务
        let task_id = manager.add_conversion_task(
            test_file.to_string_lossy().to_string(),
            DocumentType::Word,
            "test".to_string(),
            ConversionPriority::Normal,
        ).await.unwrap();

        // 检查任务状态
        let status = manager.get_task_status(task_id).await.unwrap();
        assert_eq!(status, ConversionStatus::Pending);

        // 获取统计信息
        let stats = manager.get_queue_stats().await;
        assert_eq!(stats.pending, 1);
        assert_eq!(stats.queue_size, 1);

        // 清理临时文件
        let _ = std::fs::remove_file(&test_file);
    }
}