//! 转换任务队列管理器使用示例
//! 
//! 这个示例展示了如何使用ConversionQueueManager来管理文档转换任务

use crate::import::{
    ConversionQueueManager, ConversionPriority, 
    DocumentType
};
use std::time::Duration;
use tokio::time::sleep;

/// 基本使用示例
pub async fn basic_usage_example() -> Result<(), Box<dyn std::error::Error>> {
    // 创建队列管理器，最大并发任务数为3
    let mut manager = ConversionQueueManager::new(3);
    
    // 启动队列处理器
    manager.start().await;
    
    // 添加一些转换任务
    let task1_id = manager.add_conversion_task(
        "/path/to/document1.docx".to_string(),
        DocumentType::Word,
        "Document 1".to_string(),
        ConversionPriority::High,
    ).await?;
    
    let task2_id = manager.add_conversion_task(
        "/path/to/document2.pdf".to_string(),
        DocumentType::Pdf,
        "Document 2".to_string(),
        ConversionPriority::Normal,
    ).await?;
    
    let task3_id = manager.add_conversion_task(
        "/path/to/document3.docx".to_string(),
        DocumentType::Word,
        "Document 3".to_string(),
        ConversionPriority::Low,
    ).await?;
    
    // 获取进度接收器
    let mut progress_receiver = manager.take_progress_receiver()
        .ok_or("Failed to get progress receiver")?;
    
    // 监听进度更新
    tokio::spawn(async move {
        while let Some(update) = progress_receiver.recv().await {
            println!("Task {}: {:?} - {}%", 
                update.task_id, 
                update.status, 
                update.progress
            );
            
            if let Some(error) = update.error_message {
                println!("Error: {}", error);
            }
        }
    });
    
    // 等待任务完成
    loop {
        let stats = manager.get_queue_stats().await;
        println!("Queue stats: {:?}", stats);
        
        if stats.processing == 0 && stats.pending == 0 {
            break;
        }
        
        sleep(Duration::from_secs(1)).await;
    }
    
    // 检查任务状态
    let task1_status = manager.get_task_status(task1_id).await;
    let task2_status = manager.get_task_status(task2_id).await;
    let task3_status = manager.get_task_status(task3_id).await;
    
    println!("Task 1 status: {:?}", task1_status);
    println!("Task 2 status: {:?}", task2_status);
    println!("Task 3 status: {:?}", task3_status);
    
    // 停止队列处理器
    manager.stop();
    
    Ok(())
}

/// 高级使用示例
pub async fn advanced_usage_example() -> Result<(), Box<dyn std::error::Error>> {
    let manager = ConversionQueueManager::new(2);
    manager.start().await;
    
    // 设置最大并发任务数
    manager.set_max_concurrent_tasks(4);
    
    // 添加多个任务
    let mut task_ids = Vec::new();
    for i in 1..=10 {
        let task_id = manager.add_conversion_task(
            format!("/path/to/document{}.docx", i),
            DocumentType::Word,
            format!("Document {}", i),
            if i % 3 == 0 { 
                ConversionPriority::High 
            } else { 
                ConversionPriority::Normal 
            },
        ).await?;
        task_ids.push(task_id);
    }
    
    // 取消一些任务
    if let Some(task_id) = task_ids.get(5) {
        manager.cancel_task(*task_id).await?;
        println!("Cancelled task: {}", task_id);
    }
    
    // 暂停和恢复队列
    manager.pause();
    println!("Queue paused");
    
    sleep(Duration::from_secs(2)).await;
    
    manager.resume();
    println!("Queue resumed");
    
    // 等待所有任务完成
    loop {
        let stats = manager.get_queue_stats().await;
        if stats.processing == 0 && stats.pending == 0 {
            break;
        }
        sleep(Duration::from_millis(500)).await;
    }
    
    // 清理已完成的任务（保留最近1小时的任务）
    manager.cleanup_completed_tasks(1).await;
    
    // 获取最终统计信息
    let final_stats = manager.get_queue_stats().await;
    println!("Final stats: {:?}", final_stats);
    
    manager.stop();
    Ok(())
}

/// 错误处理示例
pub async fn error_handling_example() -> Result<(), Box<dyn std::error::Error>> {
    let manager = ConversionQueueManager::new(1);
    manager.start().await;
    
    // 尝试添加不存在的文件
    let result = manager.add_conversion_task(
        "/nonexistent/file.docx".to_string(),
        DocumentType::Word,
        "Nonexistent Document".to_string(),
        ConversionPriority::Normal,
    ).await;
    
    match result {
        Ok(_) => println!("Task added successfully"),
        Err(e) => println!("Failed to add task: {}", e),
    }
    
    // 尝试取消不存在的任务
    let fake_id = uuid::Uuid::new_v4();
    let result = manager.cancel_task(fake_id).await;
    
    match result {
        Ok(_) => println!("Task cancelled successfully"),
        Err(e) => println!("Failed to cancel task: {}", e),
    }
    
    manager.stop();
    Ok(())
}

#[cfg(test)]
mod example_tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_basic_usage() {
        // 创建临时文件
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.docx");
        std::fs::write(&test_file, b"test content").unwrap();
        
        let manager = ConversionQueueManager::new(1);
        manager.start().await;
        
        // 添加任务
        let task_id = manager.add_conversion_task(
            test_file.to_string_lossy().to_string(),
            DocumentType::Word,
            "Test Document".to_string(),
            ConversionPriority::Normal,
        ).await.unwrap();
        
        // 等待任务处理
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // 检查状态
        let status = manager.get_task_status(task_id).await;
        assert!(status.is_some());
        
        manager.stop();
    }
}
