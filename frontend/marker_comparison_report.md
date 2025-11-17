# Marker 状态对比分析报告

## 基本信息

| 项目 | 本机 | 另外电脑 |
|------|------|----------|
| 主机名 | niuzhidaodeMacBook-Pro.local | macos.shared |
| 用户 | niuzhidao | niuceshi |
| 系统版本 | Darwin 25.1.0 (ARM64_T6041) | Darwin 25.1.0 (ARM64_VMAPPLE) |

---

## 🔍 关键差异分析

### 1. ✅ Marker 脚本 - **相同**

| 项目 | 本机 | 另外电脑 |
|------|------|----------|
| 路径 | `/Users/niuzhidao/Documents/Program/t1mind/frontend/resources/marker/marker` | `/Applications/AppFlowy.app/Contents/Resources/marker/marker` |
| 文件大小 | 6.7K | 6.7K |
| **文件哈希 (SHA256)** | **fcfd51b1911bbb3598489158e5a7a21fe67c2c3afc1972de71f4320bbc60e166** | **fcfd51b1911bbb3598489158e5a7a21fe67c2c3afc1972de71f4320bbc60e166** |
| 状态 | ✅ 相同 | ✅ 相同 |

**结论：** Marker 脚本内容完全相同，只是安装位置不同（开发环境 vs 应用包）。

---

### 2. ✅ marker-pdf 安装 - **基本相同**

| 项目 | 本机 | 另外电脑 |
|------|------|----------|
| pipx 版本 | 1.8.0 | 1.8.0 |
| marker-pdf 版本 | 1.10.1 | 1.10.1 |
| Python 版本 | 3.14.0 | 3.14.0 |
| marker_single 大小 | 269B | 268B |
| pipx venv 目录大小 | 1.0G | 1.0G |
| 安装路径 | `/Users/niuzhidao/.local/pipx/venvs/marker-pdf` | `/Users/niuceshi/.local/pipx/venvs/marker-pdf` |

**结论：** marker-pdf 安装状态基本相同，版本一致。

---

### 3. ⚠️ Python 环境 - **有差异**

| 项目 | 本机 | 另外电脑 |
|------|------|----------|
| 系统 Python3 | **3.14.0** (`/opt/homebrew/bin/python3`) | **3.9.6** (`/usr/bin/python3`) |
| pipx Python | 3.14.0 | 3.14.0 |

**关键差异：**
- 本机使用 Homebrew 安装的 Python 3.14.0
- 另外电脑使用系统自带的 Python 3.9.6
- 但 pipx 环境都使用 Python 3.14.0，所以 marker-pdf 运行环境一致

**影响：** 不影响 marker-pdf 运行，因为 pipx 使用独立的 Python 环境。

---

### 4. 🚨 **模型缓存目录 - 关键差异**

#### 4.1 Hugging Face 缓存 - **相同（都为空）**

| 项目 | 本机 | 另外电脑 |
|------|------|----------|
| 目录路径 | `/Users/niuzhidao/Library/Caches/huggingface` | `/Users/niuceshi/Library/Caches/huggingface` |
| 目录大小 | **0B** | **0B** |
| 文件数量 | **0** | **0** |

**结论：** 两台机器都没有下载 Hugging Face 模型。

#### 4.2 Surya OCR 模型缓存 - **⚠️ 重要差异**

| 项目 | 本机 | 另外电脑 | 差异 |
|------|------|----------|------|
| 目录路径 | `/Users/niuzhidao/Library/Caches/datalab/models` | `/Users/niuceshi/Library/Caches/datalab/models` | - |
| **目录大小** | **3.2G** | **2.7G** | **-0.5G** |
| **文件数量** | **48** | **26** | **-22 个文件** |

**详细分析：**

**本机模型文件（48 个）：**
- 包含多个模型文件，包括：
  - `model.safetensors: 1.3G` (出现多次，可能是不同模型)
  - `model.safetensors: 258M`
  - `model.safetensors: 201M`
  - `model.safetensors: 73M`
  - 以及各种配置文件和 tokenizer 文件

**另外电脑模型文件（26 个）：**
- 只包含部分模型：
  - `model.safetensors: 1.3G` (出现 2 次)
  - 缺少其他较小的模型文件（258M, 201M, 73M 等）

**可能的原因：**
1. 另外电脑的模型下载不完整
2. 本机下载了更多模型（可能是不同版本的模型或额外的模型）
3. 另外电脑可能删除了部分模型文件

**影响：** 这可能是导致另外电脑重复下载模型的原因！

---

### 5. ⚠️ 环境变量 - **相同（都未设置）**

| 环境变量 | 本机 | 另外电脑 |
|----------|------|----------|
| HF_HOME | 未设置 | 未设置 |
| HF_HUB_CACHE | 未设置 | 未设置 |
| HUGGINGFACE_HUB_CACHE | 未设置 | 未设置 |
| TRANSFORMERS_CACHE | 未设置 | 未设置 |
| HF_CACHE_DIR | ✅ 已设置 | ✅ 已设置 |
| SURYA_MODEL_CACHE_DIR | 未设置 | 未设置 |
| PYTORCH_* | 未设置 | 未设置 |

**注意：** 虽然检查时环境变量未设置，但 marker 脚本在执行时会设置这些变量。问题可能在于：
- marker 脚本执行时设置的环境变量可能没有被 marker-pdf 正确识别
- 或者模型缓存路径不一致导致 marker-pdf 认为模型不存在

---

### 6. ⚠️ 系统依赖 - **有差异**

| 依赖 | 本机 | 另外电脑 |
|------|------|----------|
| jpeg | ❌ **未安装** | ✅ 已安装 |
| libpng | ✅ 已安装 | ✅ 已安装 |
| freetype | ✅ 已安装 | ✅ 已安装 |
| openjpeg | ✅ 已安装 | ✅ 已安装 |
| libtiff | ✅ 已安装 | ✅ 已安装 |
| webp | ✅ 已安装 | ✅ 已安装 |

**影响：** 本机缺少 jpeg 依赖，但 marker-pdf 已经安装成功，说明这个依赖可能不是必需的（或者通过其他方式满足）。

---

### 7. ✅ Marker 脚本执行测试 - **都正常**

| 项目 | 本机 | 另外电脑 |
|------|------|----------|
| 执行状态 | ✅ 正常 | ✅ 正常 |
| timeout 命令 | ✅ 可用 | ❌ 不可用 |

---

## 🎯 问题诊断：为什么另外电脑会重复下载模型？

### 可能的原因：

1. **模型文件不完整（最可能）**
   - 另外电脑只有 26 个模型文件，而本机有 48 个
   - 缺少部分模型文件（特别是较小的模型：258M, 201M, 73M）
   - marker-pdf 检测到模型不完整，会尝试重新下载

2. **模型文件损坏**
   - 虽然文件存在，但可能已损坏
   - marker-pdf 验证模型完整性时发现损坏，会重新下载

3. **环境变量设置时机问题**
   - marker 脚本设置的环境变量可能在 marker-pdf 启动后才生效
   - 导致 marker-pdf 无法正确识别模型缓存路径

4. **模型缓存路径识别问题**
   - marker-pdf 可能使用不同的逻辑来识别模型缓存
   - 如果路径识别失败，会认为模型不存在

### 建议的解决方案：

#### 方案 1：复制完整的模型文件（推荐）

```bash
# 在本机上，打包模型文件
cd ~/Library/Caches/datalab
tar -czf models_backup.tar.gz models/

# 传输到另外电脑
scp models_backup.tar.gz user@other-machine:~/

# 在另外电脑上，解压并替换
cd ~/Library/Caches/datalab
tar -xzf ~/models_backup.tar.gz
```

#### 方案 2：检查 marker 脚本的环境变量设置

确保 marker 脚本在执行 marker-pdf 之前正确设置了所有环境变量，特别是：
- `HF_HOME`
- `HF_HUB_CACHE`
- `SURYA_MODEL_CACHE_DIR`

#### 方案 3：手动设置环境变量

在另外电脑上，可以在执行 marker 之前手动设置环境变量：

```bash
export HF_HOME="$HOME/Library/Caches/huggingface"
export HF_HUB_CACHE="$HOME/Library/Caches/huggingface"
export SURYA_MODEL_CACHE_DIR="$HOME/Library/Caches/datalab/models"
```

#### 方案 4：检查模型文件完整性

在另外电脑上，检查模型文件是否完整：

```bash
cd ~/Library/Caches/datalab/models
find . -name "*.safetensors" -exec ls -lh {} \;
```

---

## 📊 总结

### 相同点：
- ✅ Marker 脚本内容完全相同
- ✅ marker-pdf 版本和安装状态相同
- ✅ Hugging Face 缓存都为空
- ✅ 环境变量设置情况相同

### 关键差异：
- ⚠️ **Surya OCR 模型缓存：本机 48 个文件（3.2G），另外电脑 26 个文件（2.7G）**
- ⚠️ 系统 Python 版本不同（但不影响 marker-pdf 运行）
- ⚠️ 本机缺少 jpeg 依赖（但不影响运行）

### 最可能的问题原因：
**另外电脑的模型文件不完整，导致 marker-pdf 认为需要重新下载模型。**

### 建议：
1. **优先尝试方案 1**：从本机复制完整的模型文件到另外电脑
2. 如果问题仍然存在，检查 marker 脚本的环境变量设置逻辑
3. 确保 marker 脚本在执行时正确设置了所有必要的环境变量




