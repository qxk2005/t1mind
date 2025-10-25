# PDFium原生库集成说明

## 概述

本项目已重新设计PDF导入逻辑，直接使用PDFium原生库而不是包装库，以提供更好的多国语言支持和更准确的文本提取。

## 主要改进

### 1. 直接使用PDFium原生库
- 避免了包装库的功能限制
- 更好的性能和稳定性
- 支持最新的PDFium特性

### 2. 多国语言支持
- 使用Unicode标准化处理文本
- 支持多种编码格式（UTF-8, UTF-16LE, UTF-16BE）
- OCR支持多种语言（中文、英文、日文、韩文、阿拉伯文、俄文等）

### 3. 多层备用方案
- 优先使用PDFium原生库
- 备用lopdf库
- OCR作为最后手段

## 文件结构

```
src/import/
├── pdf_converter.rs          # 原始PDF转换器（备用）
├── pdf_converter_v2.rs       # 增强PDF转换器（备用）
├── pdf_converter_native.rs   # 原生PDFium转换器（主要）
└── converter.rs              # 转换器工厂和枚举
```

## 依赖配置

### Cargo.toml中的关键依赖

```toml
# PDF processing dependencies - using native PDFium library
lopdf = "0.32"  # Keep as fallback
html-escape = "0.2"
regex = "1.0"
xml = "0.8"
# Enhanced PDF processing dependencies
base64 = "0.22"
tempfile = "3.4"
# Unicode and text processing
unicode-normalization = "0.1"
encoding_rs = "0.8"
# Image processing for OCR fallback
image = "0.24"
# System integration and FFI
libc = "0.2"
cc = "1.0"
bindgen = "0.69"
# PDFium native library support
pdfium-sys = "0.1"
```

## PDFium库配置

### 环境变量

- `PDFIUM_PATH`: PDFium库的路径
- `PDFIUM_SRC`: PDFium源码路径（用于静态编译）

### 系统要求

#### Linux
```bash
# 安装PDFium开发包
sudo apt-get install libpdfium-dev
# 或者从源码编译
export PDFIUM_SRC=/path/to/pdfium/src
```

#### macOS
```bash
# 使用Homebrew安装
brew install pdfium
# 或者从源码编译
export PDFIUM_SRC=/path/to/pdfium/src
```

#### Windows
```cmd
# 下载PDFium预编译库
# 设置环境变量
set PDFIUM_PATH=C:\path\to\pdfium\lib
```

## 使用方法

### 基本使用

```rust
use flowy_document::import::{DefaultConverterFactory, DocumentType, ConversionConfig};

// 创建转换器工厂
let factory = DefaultConverterFactory;

// 创建PDF转换器
let converter = factory.create_converter(DocumentType::Pdf).unwrap();

// 转换PDF文件
let task = ConversionTask::new(
    "path/to/file.pdf".to_string(),
    DocumentType::Pdf,
    "converted_document".to_string(),
    ConversionConfig::default(),
);

let result = converter.convert(&task).await?;
```

### 高级配置

```rust
use flowy_document::import::{NativePdfConverter, ConversionConfig};

// 创建自定义配置
let config = ConversionConfig {
    preserve_formatting: true,
    extract_images: true,
    preserve_tables: true,
    max_file_size: Some(100 * 1024 * 1024), // 100MB
    timeout_seconds: Some(300), // 5分钟
};

// 创建原生PDF转换器
let converter = NativePdfConverter::new_with_ocr(config, true);
```

## 特性

### 1. 文本提取
- 使用PDFium原生API提取文本
- 保持原始布局和格式
- 支持复杂PDF结构

### 2. 元数据提取
- 作者信息
- 创建和修改时间
- 页面数量
- 字数统计

### 3. 多语言支持
- Unicode标准化
- 多种编码格式支持
- OCR多语言识别

### 4. 错误处理
- 详细的错误信息
- 多层备用方案
- 优雅降级

## 故障排除

### 常见问题

1. **PDFium库未找到**
   ```
   解决方案：设置PDFIUM_PATH环境变量或安装PDFium开发包
   ```

2. **编译错误**
   ```
   解决方案：确保安装了必要的系统依赖（libc, cc, bindgen）
   ```

3. **OCR失败**
   ```
   解决方案：安装Tesseract OCR引擎和语言包
   ```

### 调试模式

启用详细日志：
```rust
use tracing::Level;

tracing_subscriber::fmt()
    .with_max_level(Level::DEBUG)
    .init();
```

## 性能优化

### 1. 内存管理
- 及时释放PDFium资源
- 使用Drop trait确保清理

### 2. 并发处理
- 支持异步处理
- 线程安全的转换器

### 3. 缓存机制
- 元数据缓存
- 文本提取缓存

## 未来改进

1. **图像提取**
   - 完整的图像提取支持
   - 图像格式转换

2. **表格识别**
   - 表格结构检测
   - 表格数据提取

3. **字体处理**
   - 字体信息提取
   - 字体替换

4. **加密PDF支持**
   - 密码保护PDF
   - 权限管理

## 贡献指南

1. Fork项目
2. 创建特性分支
3. 提交更改
4. 创建Pull Request

## 许可证

本项目使用与AppFlowy相同的许可证。
