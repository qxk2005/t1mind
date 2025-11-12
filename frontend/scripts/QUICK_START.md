# Marker 状态检查脚本 - 快速开始

## 快速使用

### 1. 在正常工作的机器上生成报告

```bash
cd /path/to/frontend
./scripts/check_marker_status.sh working_report.txt
```

### 2. 在有问题的机器上生成报告

```bash
cd /path/to/frontend
./scripts/check_marker_status.sh problem_report.txt
```

### 3. 比对两个报告

```bash
# 使用 diff 比对
diff working_report.txt problem_report.txt

# 或者使用更友好的比对工具
diff -u working_report.txt problem_report.txt | less
```

## 重点关注项

比对报告时，重点关注以下差异：

1. **环境变量设置** - 确保 `HF_*` 和 `SURYA_*` 相关变量一致
2. **模型缓存目录路径** - 确保路径完全相同
3. **模型文件数量和大小** - 确保已下载的模型文件一致
4. **marker-pdf 安装路径** - 确保 pipx 安装路径一致
5. **Python 版本** - 确保 Python 版本兼容

## 常见问题快速修复

### 模型重复下载

如果发现模型重复下载，检查：

1. **环境变量是否设置：**
   ```bash
   echo $HF_HOME
   echo $SURYA_MODEL_CACHE_DIR
   ```

2. **模型缓存目录是否存在：**
   ```bash
   ls -la ~/Library/Caches/huggingface
   ls -la ~/Library/Caches/datalab/models
   ```

3. **复制模型文件（如果需要）：**
   ```bash
   # 从正常工作的机器复制模型
   scp -r user@working-machine:~/Library/Caches/huggingface ~/Library/Caches/
   scp -r user@working-machine:~/Library/Caches/datalab ~/Library/Caches/
   ```

## 输出示例

脚本会生成类似以下的输出：

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Marker 和 marker-pdf 组件检查报告
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
生成时间: 2025-01-XX XX:XX:XX
主机名: MacBook-Pro.local
...

✓ Marker 脚本找到: /path/to/marker
✓ marker-pdf 已安装
✓ Hugging Face 缓存目录存在: /Users/xxx/Library/Caches/huggingface
  目录大小: 1.2G
  文件数量: 45
✓ Surya OCR 模型缓存目录存在: /Users/xxx/Library/Caches/datalab/models
  目录大小: 2.1G
  文件数量: 12
```

保存报告文件，用于后续比对和问题诊断。


