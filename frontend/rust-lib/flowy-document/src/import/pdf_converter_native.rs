use crate::import::converter::{
    ConversionConfig, ConversionResult, ConversionTask, DocumentConverter, DocumentMetadata,
    DocumentType, DocumentContent, ConversionStatistics, ExtractedImage,
};
use flowy_error::{FlowyError, FlowyResult};
use std::path::Path;
use std::fs::File;
use std::io::Read;
use chrono::Utc;
use tracing::{info, warn, error, debug};
use image::DynamicImage;
use std::process::Command;
use std::io::Write;
use tempfile::NamedTempFile;
use unicode_normalization::UnicodeNormalization;
use encoding_rs::{Encoding, UTF_8, UTF_16LE, UTF_16BE};

/// 使用系统工具和原生库的PDF转换器
#[derive(Debug)]
pub struct NativePdfConverter {
    /// 转换配置
    config: ConversionConfig,
    /// 是否启用OCR作为备用方案
    enable_ocr_fallback: bool,
    /// 文本编码检测
    encoding_detector: EncodingDetector,
}

/// 编码检测器
#[derive(Debug)]
struct EncodingDetector {
    /// 支持的编码列表
    supported_encodings: Vec<&'static Encoding>,
}

impl EncodingDetector {
    fn new() -> Self {
        Self {
            supported_encodings: vec![UTF_8, UTF_16LE, UTF_16BE],
        }
    }
}

/// 查找系统工具的完整路径
/// 优先检查常见路径，然后尝试通过 which/where 命令查找（跨平台支持）
fn find_tool_path(command: &str) -> Option<String> {
    use std::process::Command;
    
    // 根据操作系统选择不同的路径和查找方式
    #[cfg(target_os = "windows")]
    {
        // Windows 平台
        let command_exe = if command.ends_with(".exe") {
            command.to_string()
        } else {
            format!("{}.exe", command)
        };
        
        // 检查常见的 Windows 安装路径
        let common_paths = vec![
            // Poppler 常见安装路径（通过 Chocolatey 或手动安装）
            r"C:\Program Files\poppler\bin",
            r"C:\Program Files (x86)\poppler\bin",
            r"C:\poppler\bin",
            // 用户目录下的安装
            format!(r"{}\poppler\bin", std::env::var("USERPROFILE").unwrap_or_default()),
            // 系统 PATH 中的常见路径
            r"C:\Windows\System32",
            r"C:\Windows",
            r"C:\Windows\System32\WindowsPowerShell\v1.0",
        ];
        
        for base_path in common_paths {
            let test_path = std::path::Path::new(&base_path).join(&command_exe);
            if test_path.exists() {
                debug!("Found {} at: {}", command, test_path.display());
                return Some(test_path.to_string_lossy().to_string());
            }
        }
        
        // 尝试使用 where 命令（Windows 的 which 等价物）
        if let Ok(output) = Command::new("where").arg(&command_exe).output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).lines().next()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                if let Some(path) = path_str {
                    debug!("Found {} via where: {}", command, path);
                    return Some(path);
                }
            }
        }
        
        // 尝试直接使用命令名（可能在 PATH 中）
        if let Ok(output) = Command::new(&command_exe).arg("--version").output() {
            if output.status.success() {
                debug!("Found {} in PATH", command);
                return Some(command_exe);
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // Unix-like 平台（macOS, Linux）
        // 首先检查常见的路径
        let common_paths = vec![
            "/opt/homebrew/bin",  // Apple Silicon Mac
            "/usr/local/bin",      // Intel Mac / Linux
            "/usr/bin",
            "/bin",
        ];
        
        for base_path in common_paths {
            let test_path = format!("{}/{}", base_path, command);
            if std::path::Path::new(&test_path).exists() {
                debug!("Found {} at: {}", command, test_path);
                return Some(test_path);
            }
        }
        
        // 如果常见路径中没有，尝试使用 which 命令（通过 shell）
        if let Ok(shell) = std::env::var("SHELL") {
            let which_cmd = if shell.contains("zsh") {
                format!("source ~/.zprofile 2>/dev/null || source ~/.zshrc 2>/dev/null || true; which {}", command)
            } else if shell.contains("bash") {
                format!("source ~/.bash_profile 2>/dev/null || source ~/.bashrc 2>/dev/null || true; which {}", command)
            } else {
                format!("which {}", command)
            };
            
            if let Ok(output) = Command::new(&shell)
                .arg("-c")
                .arg(&which_cmd)
                .output()
            {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path_str.is_empty() {
                        debug!("Found {} via which: {}", command, path_str);
                        return Some(path_str);
                    }
                }
            }
        }
        
        // 最后尝试直接使用 which 命令（不使用 shell）
        if let Ok(output) = Command::new("which").arg(command).output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    debug!("Found {} via which: {}", command, path_str);
                    return Some(path_str);
                }
            }
        }
    }
    
    warn!("Could not find {} in common paths or via which/where", command);
    None
}

impl EncodingDetector {
    /// 检测文本编码
    fn detect_encoding(&self, bytes: &[u8]) -> &'static Encoding {
        // 首先尝试UTF-8
        if UTF_8.decode(bytes).0.is_empty() {
            return UTF_8;
        }

        // 尝试UTF-16LE
        if bytes.len() % 2 == 0 {
            if let Ok(utf16_str) = std::str::from_utf8(bytes) {
                if utf16_str.chars().all(|c| c.is_ascii()) {
                    return UTF_16LE;
                }
            }
        }

        // 默认返回UTF-8
        UTF_8
    }

    /// 解码文本，支持多种编码
    fn decode_text(&self, bytes: &[u8]) -> String {
        let encoding = self.detect_encoding(bytes);
        let (decoded, _, _) = encoding.decode(bytes);
        decoded.to_string()
    }
}

impl NativePdfConverter {
    /// 创建新的原生PDF转换器
    pub fn new(config: ConversionConfig) -> Self {
        Self { 
            config,
            enable_ocr_fallback: true,
            encoding_detector: EncodingDetector::new(),
        }
    }

    /// 创建新的原生PDF转换器（带OCR选项）
    pub fn new_with_ocr(config: ConversionConfig, enable_ocr_fallback: bool) -> Self {
        Self { 
            config,
            enable_ocr_fallback,
            encoding_detector: EncodingDetector::new(),
        }
    }

    /// 使用系统工具提取PDF文本内容
    fn extract_text_with_system_tools(&self, pdf_data: &[u8]) -> FlowyResult<String> {
        info!("[PDF导入进度] 正在提取文本内容...");
        
        // 创建临时PDF文件
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(pdf_data)?;
        temp_file.flush()?;

        // 尝试使用系统工具提取文本
        match self.try_system_tools_extraction(temp_file.path()) {
            Ok(text) if !text.trim().is_empty() => {
                info!("Successfully extracted text using system tools: {} characters", text.len());
                return Ok(text);
            }
            Ok(_) => {
                warn!("System tools extracted empty text");
            }
            Err(e) => {
                // 如果是因为工具未安装导致的失败，记录信息但继续尝试其他方法
                let error_msg = e.to_string();
                if error_msg.contains("not found") || error_msg.contains("command not found") {
                    info!("System PDF extraction tools not available: {}. Trying fallback methods...", error_msg);
                } else {
                    warn!("System tools extraction failed: {}", e);
                }
            }
        }

        // 如果都失败了，尝试OCR
        if self.enable_ocr_fallback {
            match self.try_ocr_extraction(pdf_data) {
                Ok(text) if !text.trim().is_empty() => {
                    info!("Successfully extracted text using OCR: {} characters", text.len());
                    return Ok(text);
                }
                Ok(_) => {
                    warn!("OCR extracted empty text");
                }
                Err(e) => {
                    warn!("OCR extraction failed: {}", e);
                }
            }
        }

        Err(FlowyError::new(
            flowy_error::ErrorCode::Internal,
            "Failed to extract text using all available methods"
        ))
    }

    /// 尝试使用系统工具提取文本
    fn try_system_tools_extraction(&self, pdf_path: &Path) -> FlowyResult<String> {
        // 尝试使用pdf2txt.py
        match self.try_pdf2txt(pdf_path) {
            Ok(text) => return Ok(text),
            Err(e) => {
                debug!("pdf2txt.py extraction failed: {}", e);
                // 检查是否是命令不存在或模块不存在的问题
                if e.to_string().contains("No such file") 
                    || e.to_string().contains("not found")
                    || e.to_string().contains("ModuleNotFoundError") {
                    info!("pdfminer not available. Falling back to other methods...");
                }
            }
        }

        Err(FlowyError::new(
            flowy_error::ErrorCode::Internal,
            "System tools extraction failed"
        ))
    }


    /// 使用pdf2txt.py提取文本
    fn try_pdf2txt(&self, pdf_path: &Path) -> FlowyResult<String> {
        // Windows 使用 python，Unix-like 使用 python3
        let python_cmd = if cfg!(target_os = "windows") {
            "python"
        } else {
            "python3"
        };
        
        // 在 Windows 上，路径需要使用反斜杠，但 Python 字符串中需要转义或使用原始字符串
        let pdf_path_str = if cfg!(target_os = "windows") {
            pdf_path.to_string_lossy().replace('\\', "/")  // Python 接受正斜杠路径
        } else {
            pdf_path.to_string_lossy().to_string()
        };
        
        let output = match Command::new(python_cmd)
            .arg("-c")
            .arg(format!(
                "import sys; from pdfminer.high_level import extract_text; print(extract_text(r'{}'))",
                pdf_path_str
            ))
            .output()
        {
            Ok(output) => output,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return Err(FlowyError::new(
                        flowy_error::ErrorCode::Internal,
                        {
                            #[cfg(target_os = "windows")]
                            {
                                format!("python command not found: {}. Install Python and pdfminer.six (pip install pdfminer.six) for PDF text extraction.", e)
                            }
                            #[cfg(not(target_os = "windows"))]
                            {
                                format!("python3 command not found: {}. Install python3 and pdfminer.six (pip install pdfminer.six) for PDF text extraction.", e)
                            }
                        }
                    ));
                }
                return Err(FlowyError::from(e));
            }
        };

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            // 检查是否是模块未安装的错误
            if error_msg.contains("ModuleNotFoundError") || error_msg.contains("No module named") {
                return Err(FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("pdfminer.six not installed: {}. Install it with: pip install pdfminer.six", error_msg)
                ));
            }
            return Err(FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("pdf2txt.py failed: {}", error_msg)
            ));
        }

        let text = String::from_utf8_lossy(&output.stdout).to_string();
        
        // 直接返回原始文本，不做任何清理和优化
        Ok(text)
    }

    /// 清理提取的文本，移除无效内容
    /// 注意：保留空行作为段落分隔符，以维持原始段落结构
    fn clean_extracted_text(&self, text: &str) -> String {
        text.lines()
            .map(|line| {
                let trimmed = line.trim();
                
                // 过滤掉只包含 Identity-H 错误的行
                if trimmed.contains("Identity-H") || trimmed.contains("Unimplemented") {
                    return String::new();
                }
                
                // 过滤掉全是问号的行
                if trimmed.len() > 10 && trimmed.chars().all(|c| c == '?') {
                    return String::new();
                }
                
                // 过滤掉包含大量乱码字符的行
                if !trimmed.is_empty() && self.is_line_garbled(trimmed) {
                    return String::new();
                }
                
                // 保留原始行（包括空行），以维持段落结构
                line.to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 检测单行文本是否主要是乱码
    fn is_line_garbled(&self, line: &str) -> bool {
        // 跳过 "Page X:" 这样的行
        if line.trim().starts_with("Page ") && line.trim().ends_with(":") {
            return false;
        }
        
        if line.len() < 5 {
            return false; // 太短的行不判断为乱码
        }

        let chars: Vec<char> = line.chars().collect();
        let mut garbled_count = 0;
        let mut printable_count = 0;
        let mut chinese_count = 0;
        let mut ascii_letter_count = 0;
        
        for &ch in &chars {
            // 控制字符（排除正常的换行符等）
            if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
                garbled_count += 1;
            }
            // 中文字符
            else if (ch as u32) >= 0x4E00 && (ch as u32) <= 0x9FFF {
                chinese_count += 1;
                printable_count += 1;
            }
            // ASCII 字母数字
            else if ch.is_ascii_alphanumeric() {
                printable_count += 1;
                if ch.is_ascii_alphabetic() {
                    ascii_letter_count += 1;
                }
            }
            // 常见标点和空格
            else if ch.is_whitespace() || ['.', ',', ';', ':', '!', '?', '-', '_', '(', ')', '[', ']', '{', '}', '"', '\'', '/', '\\', '=', '+', '*', '&', '%', '$', '#', '@', '^', '|', '<', '>'].contains(&ch) {
                printable_count += 1;
            }
            // 其他 Unicode 字符
            else if (ch as u32) > 127 {
                let code = ch as u32;
                // 常见东亚字符范围和其他常见 Unicode 范围
                if (code >= 0x3040 && code <= 0x309F) || // 日文平假名
                   (code >= 0x30A0 && code <= 0x30FF) || // 日文片假名
                   (code >= 0xAC00 && code <= 0xD7AF) || // 韩文
                   (code >= 0x2000 && code <= 0x206F) || // 常用标点
                   (code >= 0x2070 && code <= 0x209F) || // 上标下标
                   (code >= 0x20A0 && code <= 0x20CF) || // 货币符号
                   (code >= 0x2100 && code <= 0x214F) || // 字母符号
                   (code >= 0x2190 && code <= 0x21FF) {  // 箭头
                    printable_count += 1;
                } else if code >= 0xE000 && code <= 0xF8FF {
                    // 私有使用区域，很可能是乱码
                    garbled_count += 1;
                } else if code < 0x100 {
                    // 其他低值 Unicode 字符可能是乱码
                    garbled_count += 1;
                } else {
                    // 其他 Unicode 字符，保守地认为可能是有效的
                    printable_count += 1;
                }
            } else {
                printable_count += 1;
            }
        }

        let total = chars.len();
        
        // 如果包含足够的中文字符或 ASCII 字母，认为是有效文本
        if chinese_count > total / 20 || ascii_letter_count > total / 5 {
            return false;
        }
        
        // 如果乱码字符超过60%（提高阈值），才认为是乱码行
        if total > 0 && (garbled_count as f32 / total as f32) > 0.6 {
            return true;
        }
        
        // 如果可打印字符少于20%（降低阈值），认为是乱码行
        if total > 0 && (printable_count as f32 / total as f32) < 0.2 {
            return true;
        }

        false
    }

    /// 验证提取的文本是否有效（不全是 Identity-H 错误或乱码）
    fn is_text_valid(&self, text: &str) -> bool {
        let cleaned = self.clean_extracted_text(text);
        let cleaned_len = cleaned.trim().len();
        let original_len = text.trim().len();
        
        // 如果清理后的文本长度小于原文本的20%，说明大部分是无效内容
        if original_len == 0 {
            return false;
        }
        
        let valid_ratio = cleaned_len as f32 / original_len as f32;
        
        // 检查是否包含乱码字符
        if self.contains_garbled_text(&cleaned) {
            return false;
        }
        
        // 至少要有30%的有效内容，并且至少要有一些非空内容
        valid_ratio > 0.3 && cleaned_len > 50
    }

    /// 检测文本是否包含大量乱码字符
    fn contains_garbled_text(&self, text: &str) -> bool {
        if text.trim().is_empty() {
            return true;
        }

        let chars: Vec<char> = text.chars().collect();
        let total_chars = chars.len();
        if total_chars == 0 {
            return true;
        }

        let mut garbled_count = 0;
        let mut printable_count = 0;
        let mut chinese_char_count = 0;
        let mut ascii_letter_count = 0;
        
        for &ch in &chars {
            if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
                garbled_count += 1;
            } else if ch.is_alphabetic() || ch.is_alphanumeric() {
                printable_count += 1;
                // 检测中文字符
                if (ch as u32) >= 0x4E00 && (ch as u32) <= 0x9FFF {
                    chinese_char_count += 1;
                } else if ch.is_ascii_alphabetic() {
                    ascii_letter_count += 1;
                }
            } else if ch.is_whitespace() || ['.', ',', ';', ':', '!', '?', '-', '(', ')', '[', ']', '{', '}', '"', '\''].contains(&ch) {
                printable_count += 1;
            } else if (ch as u32) > 127 {
                // 非 ASCII 字符，可能是中文、日文、韩文等，也可能是乱码
                // 检查是否是常见 Unicode 范围
                let code = ch as u32;
                if (code >= 0x4E00 && code <= 0x9FFF) || // 中文
                   (code >= 0x3040 && code <= 0x309F) || // 日文平假名
                   (code >= 0x30A0 && code <= 0x30FF) || // 日文片假名
                   (code >= 0xAC00 && code <= 0xD7AF) {  // 韩文
                    printable_count += 1;
                } else if code < 0x1000 {
                    // 其他低值 Unicode 字符可能是乱码
                    garbled_count += 1;
                }
            } else {
                // 其他 ASCII 字符（可能是特殊符号，也可能是乱码）
                // 如果连续出现大量特殊字符，可能是乱码
                printable_count += 1;
            }
        }

        // 如果包含中文字符或大量可打印字符，可能是有效文本
        if chinese_char_count > 10 || ascii_letter_count > total_chars / 4 {
            // 包含足够的中文或英文，即使有一些乱码字符也认为是有效的
            return false;
        }

        // 如果乱码字符超过50%（提高阈值），认为文本无效
        let garbled_ratio = garbled_count as f32 / total_chars as f32;
        if garbled_ratio > 0.5 {
            return true;
        }

        // 如果可打印字符少于15%（降低阈值），认为文本无效
        let printable_ratio = printable_count as f32 / total_chars as f32;
        if printable_ratio < 0.15 {
            return true;
        }

        // 检测是否包含大量连续的特殊字符组合（乱码的典型特征）
        let mut consecutive_special = 0;
        let mut max_consecutive_special = 0;
        for &ch in &chars {
            if !ch.is_alphanumeric() && !ch.is_whitespace() && 
               !['.', ',', ';', ':', '!', '?', '-', '_', '(', ')', '[', ']', '{', '}', '"', '\'', '/', '\\', '=', '+', '*', '&', '%', '$', '#', '@'].contains(&ch) {
                consecutive_special += 1;
                max_consecutive_special = max_consecutive_special.max(consecutive_special);
            } else {
                consecutive_special = 0;
            }
        }

        // 如果连续特殊字符超过100个（提高阈值），可能是乱码
        if max_consecutive_special > 100 {
            return true;
        }

        false
    }

    /// 尝试使用OCR提取文本
    fn try_ocr_extraction(&self, pdf_data: &[u8]) -> FlowyResult<String> {
        info!("Attempting OCR extraction as fallback");
        
        // 将PDF转换为图像
        let image = self.pdf_to_image(pdf_data)?;
        
        // 使用OCR提取文本
        let ocr_text = self.perform_ocr(&image)?;
        
        Ok(ocr_text)
    }

    /// 将PDF转换为图像
    fn pdf_to_image(&self, pdf_data: &[u8]) -> FlowyResult<DynamicImage> {
        let mut temp_pdf = NamedTempFile::new()?;
        temp_pdf.write_all(pdf_data)?;
        temp_pdf.flush()?;

        // 使用pdftoppm转换为图像
        let output_dir = tempfile::tempdir()?;
        let output_prefix = output_dir.path().join("page");
        
        // 查找 pdftoppm 的完整路径
        let pdftoppm_path = find_tool_path("pdftoppm")
            .ok_or_else(|| FlowyError::new(
                flowy_error::ErrorCode::Internal,
                {
                    #[cfg(target_os = "windows")]
                    {
                        "pdftoppm command not found. Install poppler-utils (choco install poppler or download from https://github.com/oschwartz10612/poppler-windows/releases/) for OCR support.".to_string()
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        "pdftoppm command not found. Install poppler-utils (brew install poppler on macOS, or apt-get install poppler-utils on Linux) for OCR support.".to_string()
                    }
                }
            ))?;
        
        let output = match Command::new(&pdftoppm_path)
            .arg("-png")
            .arg("-r")
            .arg("300") // 300 DPI for better OCR
            .arg("-f")
            .arg("1")
            .arg("-l")
            .arg("1")
            .arg(temp_pdf.path())
            .arg(&output_prefix)
            .output()
        {
            Ok(output) => output,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return Err(FlowyError::new(
                        flowy_error::ErrorCode::Internal,
                        {
                            #[cfg(target_os = "windows")]
                            {
                                format!("pdftoppm command not found: {}. Install poppler-utils (choco install poppler or download from https://github.com/oschwartz10612/poppler-windows/releases/) for OCR support.", e)
                            }
                            #[cfg(not(target_os = "windows"))]
                            {
                                format!("pdftoppm command not found: {}. Install poppler-utils (brew install poppler on macOS, or apt-get install poppler-utils on Linux) for OCR support.", e)
                            }
                        }
                    ));
                }
                return Err(FlowyError::from(e));
            }
        };

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(FlowyError::new(
                flowy_error::ErrorCode::Internal,
                {
                    #[cfg(target_os = "windows")]
                    {
                        format!("pdftoppm failed: {}. Make sure poppler-utils is installed (choco install poppler or download from https://github.com/oschwartz10612/poppler-windows/releases/).", error_msg)
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        format!("pdftoppm failed: {}. Make sure poppler-utils is installed (brew install poppler on macOS, or apt-get install poppler-utils on Linux).", error_msg)
                    }
                }
            ));
        }

        // 查找生成的图片文件
        let image_path = output_dir.path().join("page-1.png");
        if image_path.exists() {
            let image_data = std::fs::read(&image_path)?;
            let image = image::load_from_memory(&image_data)
                .map_err(|e| FlowyError::new(flowy_error::ErrorCode::Internal, format!("Failed to load image: {}", e)))?;
            return Ok(image);
        }

        Err(FlowyError::new(
            flowy_error::ErrorCode::Internal,
            "pdftoppm did not generate expected output file"
        ))
    }

    /// 执行OCR
    fn perform_ocr(&self, image: &DynamicImage) -> FlowyResult<String> {
        // 将图像保存到临时文件
        let mut temp_image = NamedTempFile::with_prefix("ocr_image_")?;
        image.write_to(&mut temp_image, image::ImageFormat::Png)
            .map_err(|e| FlowyError::new(flowy_error::ErrorCode::Internal, format!("Failed to write image: {}", e)))?;
        temp_image.flush()?;

        // 使用Tesseract进行OCR，支持多语言
        // 查找 tesseract 的完整路径
        let tesseract_path = find_tool_path("tesseract")
            .ok_or_else(|| FlowyError::new(
                flowy_error::ErrorCode::Internal,
                "tesseract command not found. Install tesseract (brew install tesseract tesseract-lang) for OCR support.".to_string()
            ))?;
        
        let output = match Command::new(&tesseract_path)
            .arg(temp_image.path())
            .arg("stdout")
            .arg("-l")
            .arg("chi_sim+eng+jpn+kor+ara+rus") // 支持多种语言
            .arg("--psm")
            .arg("6") // 统一文本块
            .arg("-c")
            .arg("preserve_interword_spaces=1") // 保持单词间距
            .output()
        {
            Ok(output) => output,
            Err(e) => {
                return Err(FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("Tesseract OCR execution failed: {}. Path used: {}", e, tesseract_path)
                ));
            }
        };

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("Tesseract OCR failed: {}", error_msg);
            
            // 检查是否是语言包缺失的问题
            if error_msg.contains("chi_sim") || error_msg.contains("language") {
                warn!("Chinese language pack may not be installed. Try: brew install tesseract-lang");
            }
            
            return Err(FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("Tesseract OCR failed: {}", error_msg)
            ));
        }

        let ocr_text = String::from_utf8_lossy(&output.stdout).to_string();
        
        // 直接返回原始OCR文本，不做任何清理和优化
        Ok(ocr_text)
    }

    /// 清理OCR文本
    fn clean_ocr_text(&self, text: &str) -> String {
        let mut cleaned = text.to_string();
        
        // 移除多余的空白字符
        cleaned = cleaned.replace("\r\n", "\n");
        cleaned = cleaned.replace("\r", "\n");
        
        // 移除行首行尾的空白
        let lines: Vec<&str> = cleaned.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();
        
        lines.join("\n")
    }

    /// 标准化文本，处理Unicode
    fn normalize_text(&self, text: &str) -> String {
        // 使用Unicode标准化
        text.nfc().collect::<String>()
    }

    /// 提取PDF文档元数据
    async fn extract_document_metadata(&self, _pdf_data: &[u8], _file_path: &str) -> FlowyResult<DocumentMetadata> {
        let mut metadata = DocumentMetadata {
            author: None,
            created_at: None,
            modified_at: None,
            page_count: None,
            word_count: None,
            language: None,
        };


        // 如果无法从PDF中获取创建时间，使用当前时间
        if metadata.created_at.is_none() || metadata.modified_at.is_none() {
            let now = Utc::now();
            if metadata.created_at.is_none() {
                metadata.created_at = Some(now);
            }
            if metadata.modified_at.is_none() {
                metadata.modified_at = Some(now);
            }
        }

        Ok(metadata)
    }
    
    /// 检测是否为标题
    fn detect_heading(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        // 跳过页面标记
        if trimmed.starts_with("Page ") && trimmed.ends_with(":") {
            return false;
        }
        
        // 如果行很短（少于50字符）且看起来像标题
        if trimmed.len() < 50 {
            // 检查是否全大写（可能是英文标题）
            if trimmed.chars().all(|c| !c.is_lowercase() && c.is_alphabetic()) && trimmed.len() > 3 {
                return true;
            }
            
            // 检查是否以数字开头（如 "1. 概述"）
            if trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                // 检查是否只有很少的标点符号
                let punctuation_count = trimmed.chars().filter(|c| ['.', ':', '、', '。'].contains(c)).count();
                if punctuation_count <= 2 && trimmed.len() < 30 {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// 确定标题级别（1-6）
    fn determine_heading_level(&self, line: &str) -> u32 {
        let trimmed = line.trim();
        
        // 如果以多个#开头，提取级别
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            return level.min(6) as u32;
        }
        
        // 根据行长度和内容判断
        // 短行（<20字符）可能是h1或h2
        if trimmed.len() < 20 {
            return 2;
        } else if trimmed.len() < 40 {
            return 3;
        } else {
            return 4;
        }
    }
    
    /// 检测是否为列表项
    /// 返回 (是否为列表项, 是否为有序列表)
    fn detect_list_item(&self, line: &str) -> (bool, bool) {
        let trimmed = line.trim();
        
        // 检查无序列表标记
        if trimmed.starts_with("• ") || 
           trimmed.starts_with("- ") || 
           trimmed.starts_with("* ") ||
           trimmed.starts_with("○ ") {
            return (true, false);
        }
        
        // 检查有序列表标记（数字. 或 数字、）
        if let Some(first_char) = trimmed.chars().next() {
            if first_char.is_ascii_digit() {
                // 检查是否以 "数字. " 或 "数字、" 开头
                let pattern_ordered = format!("{}. ", first_char);
                let pattern_ordered_chinese = format!("{}、", first_char);
                
                if trimmed.starts_with(&pattern_ordered) || trimmed.starts_with(&pattern_ordered_chinese) {
                    // 检查是否真的是有序列表（数字应该递增，但这里简单判断）
                    return (true, true);
                }
            }
        }
        
        // 检查中文字符后的特殊标记
        if trimmed.starts_with("（") && trimmed.chars().nth(1).map(|c| c.is_ascii_digit()).unwrap_or(false) {
            return (true, true);
        }
        
        (false, false)
    }
    
    /// 提取列表项文本（移除列表标记）
    fn extract_list_item_text(&self, line: &str) -> String {
        let trimmed = line.trim();
        
        // 移除无序列表标记
        if trimmed.starts_with("• ") {
            return trimmed[2..].trim().to_string();
        }
        if trimmed.starts_with("- ") {
            return trimmed[2..].trim().to_string();
        }
        if trimmed.starts_with("* ") {
            return trimmed[2..].trim().to_string();
        }
        if trimmed.starts_with("○ ") {
            return trimmed[2..].trim().to_string();
        }
        
        // 移除有序列表标记（如 "1. " 或 "1、"）
        if let Some(first_char) = trimmed.chars().next() {
            if first_char.is_ascii_digit() {
                // 尝试匹配 "数字. " 或 "数字、"
                if let Some(rest) = trimmed.strip_prefix(&format!("{}. ", first_char)) {
                    return rest.trim().to_string();
                }
                if let Some(rest) = trimmed.strip_prefix(&format!("{}、", first_char)) {
                    return rest.trim().to_string();
                }
                // 匹配更复杂的模式，如 "1.1. " 或 "（1）"
                if trimmed.starts_with("（") {
                    if let Some(pos) = trimmed.find('）') {
                        return trimmed[pos+1..].trim().to_string();
                    }
                }
            }
        }
        
        trimmed.to_string()
    }

    /// 在HTML中根据图片位置插入占位符
    /// 这个方法会在HTML转Markdown之前调用，在HTML中插入图片占位符
    fn insert_image_placeholders_into_html(&self, html_content: &str, images: &[ExtractedImage]) -> String {
        use regex::Regex;
        
        // 如果HTML中已经有img标签，不需要插入占位符
        if html_content.contains(r#"<img"#) || html_content.contains(r#"<IMG"#) {
            info!("HTML中已有img标签，不需要插入占位符");
            return html_content.to_string();
        }
        
        if images.is_empty() {
            return html_content.to_string();
        }
        
        info!("[图片占位符] 在HTML中插入 {} 个图片占位符", images.len());
        
        // 提取所有段落的位置信息（top position）
        let p_regex = Regex::new(r#"(?s)<p[^>]*style="[^"]*top:\s*(\d+(?:\.\d+)?)px[^"]*"[^>]*>([^<]+)</p>"#).unwrap();
        let mut paragraphs_with_pos: Vec<(f64, usize, String)> = Vec::new(); // (top_pos, original_index, text)
        
        for (idx, mat) in p_regex.find_iter(html_content).enumerate() {
            if let Some(caps) = p_regex.captures(mat.as_str()) {
                if let (Some(top_match), Some(text_match)) = (caps.get(1), caps.get(2)) {
                    if let Ok(top_pos) = top_match.as_str().parse::<f64>() {
                        let text = text_match.as_str().trim();
                        if !text.is_empty() {
                            paragraphs_with_pos.push((top_pos, mat.start(), text.to_string()));
                        }
                    }
                }
            }
        }
        
        // 按位置排序
        paragraphs_with_pos.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        
        info!("[图片占位符] 找到 {} 个带位置的段落", paragraphs_with_pos.len());
        
        // 根据图片数量和段落位置，在合适的位置插入占位符
        // 策略：根据PDF中的逻辑顺序，在标题段落之后插入图片
        // 例如：如果"弹性算力"是标题（top位置靠后），在第一段描述性文字之后、标题之前插入图片
        let mut result = html_content.to_string();
        
        // 收集所有需要插入的占位符信息（位置和内容）
        let mut insertions: Vec<(usize, String)> = Vec::new(); // (position, placeholder_html)
        
        // 找到标题段落（H1或H2标签，或者font-size较大的段落）
        // 策略：在第一个标题段落之前插入第一张图片，在第二个标题段落之前插入第二张图片
        let h1_regex = Regex::new(r#"(?s)<h1[^>]*>([^<]+)</h1>"#).unwrap();
        let h2_regex = Regex::new(r#"(?s)<h2[^>]*>([^<]+)</h2>"#).unwrap();
        
        // 先检查是否有标题标签
        let mut heading_positions: Vec<(usize, String, usize)> = Vec::new(); // (position, text, heading_level)
        
        for mat in h1_regex.find_iter(&result) {
            if let Some(caps) = h1_regex.captures(mat.as_str()) {
                if let Some(text_match) = caps.get(1) {
                    heading_positions.push((mat.start(), text_match.as_str().to_string(), 1));
                }
            }
        }
        
        for mat in h2_regex.find_iter(&result) {
            if let Some(caps) = h2_regex.captures(mat.as_str()) {
                if let Some(text_match) = caps.get(1) {
                    heading_positions.push((mat.start(), text_match.as_str().to_string(), 2));
                }
            }
        }
        
        // 按位置排序
        heading_positions.sort_by(|a, b| a.0.cmp(&b.0));
        
        info!("[图片占位符] 找到 {} 个标题段落", heading_positions.len());
        
        // 如果没有找到标题标签，尝试根据段落位置推断
        // 在"弹性算力"这样的关键词之后插入图片
        if heading_positions.is_empty() && !paragraphs_with_pos.is_empty() {
            // 查找包含"弹性算力"的段落
            let mut target_para_idx = None;
            for (idx, (_top_pos, _para_start, para_text)) in paragraphs_with_pos.iter().enumerate() {
                if para_text.contains("弹性算力") || para_text.contains("弹性") {
                    target_para_idx = Some(idx);
                    info!("[图片占位符] 找到包含'弹性算力'的段落: {}", para_text.chars().take(30).collect::<String>());
                    break;
                }
            }
            
            // 在目标段落之后插入图片
            if let Some(target_idx) = target_para_idx {
                // 找到目标段落在HTML中的位置（查找H1标签或段落标签）
                let (_, _target_start, target_text) = &paragraphs_with_pos[target_idx];
                // 先尝试查找H1标签
                let h1_pattern = format!(r#"(?s)<h1[^>]*>([^<]*{})</h1>"#, regex::escape(target_text));
                let h1_match = Regex::new(&h1_pattern).ok().and_then(|r| r.find(&result));
                
                if let Some(h1_mat) = h1_match {
                    // 在H1标签之后插入
                    if images.len() > 0 {
                        let placeholder = format!(
                            r#"
<img data-placeholder="IMAGE_PLACEHOLDER_1" alt="图片 1" style="display: block; margin: 10px auto;" />"#,
                        );
                        insertions.push((h1_mat.end(), placeholder));
                        info!("[图片占位符] 在'弹性算力'H1标题之后插入图片 1 的占位符");
                    }
                } else {
                    // 查找段落标签
                    let target_pattern = format!(r#"<p[^>]*>([^<]*{})</p>"#, regex::escape(target_text));
                    if let Some(target_match) = Regex::new(&target_pattern).ok().and_then(|r| r.find(&result)) {
                        // 在段落之后插入
                        if images.len() > 0 {
                            let placeholder = format!(
                                r#"
<img data-placeholder="IMAGE_PLACEHOLDER_1" alt="图片 1" style="display: block; margin: 10px auto;" />"#,
                            );
                            insertions.push((target_match.end(), placeholder));
                            info!("[图片占位符] 在'弹性算力'段落之后插入图片 1 的占位符");
                        }
                    }
                }
            }
        } else {
            // 根据标题位置插入图片（在标题之后）
            for (img_idx, _image) in images.iter().enumerate() {
                if img_idx < heading_positions.len() {
                    let (heading_pos, heading_text, _heading_level) = &heading_positions[img_idx];
                    // 查找标题标签的结束位置
                    let heading_end_pattern = format!(r#"(?s)<h[12][^>]*>([^<]*{})</h[12]>"#, regex::escape(heading_text));
                    if let Some(heading_match) = Regex::new(&heading_end_pattern).ok().and_then(|r| r.find(&result)) {
                        let placeholder = format!(
                            r#"
<img data-placeholder="IMAGE_PLACEHOLDER_{}" alt="图片 {}" style="display: block; margin: 10px auto;" />"#,
                            img_idx + 1,
                            img_idx + 1
                        );
                        insertions.push((heading_match.end(), placeholder));
                        info!("[图片占位符] 在第 {} 个标题之后插入图片 {} 的占位符", img_idx + 1, img_idx + 1);
                    }
                }
            }
        }
        
        // 如果还有图片没有插入，在第一个段落之后插入
        if images.len() > insertions.len() {
            if let Some(first_para) = paragraphs_with_pos.first() {
                let (_, _first_para_start, first_para_text) = first_para;
                let first_para_pattern = format!(r#"<p[^>]*>([^<]*{})</p>"#, regex::escape(first_para_text));
                if let Some(first_para_match) = Regex::new(&first_para_pattern).ok().and_then(|r| r.find(&result)) {
                    for img_idx in insertions.len()..images.len() {
                        let placeholder = format!(
                            r#"
<img data-placeholder="IMAGE_PLACEHOLDER_{}" alt="图片 {}" style="display: block; margin: 10px auto;" />"#,
                            img_idx + 1,
                            img_idx + 1
                        );
                        insertions.push((first_para_match.end(), placeholder));
                        info!("[图片占位符] 在第一段之后插入图片 {} 的占位符", img_idx + 1);
                    }
                }
            }
        }
        
        // 按位置倒序插入，避免位置偏移
        insertions.sort_by(|a, b| b.0.cmp(&a.0));
        for (pos, placeholder) in insertions {
            result.insert_str(pos, &placeholder);
        }
        
        result
    }
    
    /// 将图片插入到 HTML 中的适当位置
    /// 尝试按页面顺序和文本结构插入图片，而不是全部追加到末尾
    fn insert_images_into_html(&self, html_content: &str, images: &[ExtractedImage]) -> String {
        use regex::Regex;
        use base64::{Engine as _, engine::general_purpose};
        
        // 首先检查是否有占位符
        let placeholder_regex = Regex::new(r#"<img[^>]+data-placeholder="IMAGE_PLACEHOLDER_(\d+)"[^>]*>"#)
            .unwrap();
        
        let placeholders: Vec<usize> = placeholder_regex.captures_iter(html_content)
            .filter_map(|cap| {
                cap.get(1)
                    .and_then(|m| m.as_str().parse::<usize>().ok())
            })
            .collect();
        
        if !placeholders.is_empty() {
            info!("Found {} image placeholders, replacing with extracted images", placeholders.len());
            
            // 按占位符顺序替换
            let mut result = html_content.to_string();
            let mut image_idx = 0;
            
            // 按占位符编号排序（虽然应该已经是顺序的，但为了安全）
            let mut sorted_placeholders: Vec<(usize, String)> = placeholders.iter()
                .map(|&placeholder_num| {
                    // 找到对应的占位符标签
                    let placeholder_pattern = format!(r#"data-placeholder="IMAGE_PLACEHOLDER_{}""#, placeholder_num);
                    let placeholder_tag_regex = Regex::new(&format!(
                        r#"<img[^>]*{}[^>]*>"#,
                        regex::escape(&placeholder_pattern)
                    )).unwrap();
                    
                    let full_tag = placeholder_tag_regex.find(html_content)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default();
                    
                    (placeholder_num, full_tag)
                })
                .collect();
            
            // 按编号排序
            sorted_placeholders.sort_by_key(|(num, _)| *num);
            
            // 替换每个占位符
            for (placeholder_num, placeholder_tag) in sorted_placeholders {
                if image_idx >= images.len() {
                    warn!("More placeholders than images, skipping placeholder {}", placeholder_num);
                    continue;
                }
                
                let image = &images[image_idx];
                let base64_data = general_purpose::STANDARD.encode(&image.data);
                let mime_type = match image.format {
                    crate::import::converter::ImageFormat::Png => "image/png",
                    crate::import::converter::ImageFormat::Jpeg => "image/jpeg",
                    crate::import::converter::ImageFormat::Gif => "image/gif",
                    crate::import::converter::ImageFormat::Bmp => "image/bmp",
                    crate::import::converter::ImageFormat::WebP => "image/webp",
                };
                let data_url = format!("data:{};base64,{}", mime_type, base64_data);
                
                let (width, height) = image.dimensions.as_ref()
                    .map(|d| (d.width, d.height))
                    .unwrap_or((300, 200));
                
                // 提取原有style（如果有）
                let original_style = Regex::new(r#"style=["']([^"']+)["']"#)
                    .ok()
                    .and_then(|r| r.captures(&placeholder_tag))
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| "max-width: 100%; height: auto;".to_string());
                
                let image_html = format!(
                    r#"<img src="{}" alt="{}" width="{}" height="{}" style="{}" />"#,
                    data_url, image.filename, width, height, original_style
                );
                
                result = result.replacen(&placeholder_tag, &image_html, 1);
                info!("Replaced placeholder {} with image {} ({}x{})", 
                    placeholder_num, image.filename, width, height);
                image_idx += 1;
            }
            
            return result;
        }
        
        // 如果没有占位符，使用原来的逻辑
        // 检查 HTML 中是否有页面标记（如 "Page X:"）
        let has_page_markers = html_content.contains("Page ") && html_content.contains(":");
        
        if has_page_markers {
            // 方法1: 如果有页面标记，尝试在每个页面标记后插入对应页面的图片
            // 注意：pdfimages 提取的图片顺序可能与页面顺序对应
            return self.insert_images_by_page_markers(html_content, images);
        } else {
            // 方法2: 如果没有页面标记，将图片均匀分布在整个文档中
            return self.insert_images_evenly(html_content, images);
        }
    }
    
    /// 按页面标记插入图片
    fn insert_images_by_page_markers(&self, html_content: &str, images: &[ExtractedImage]) -> String {
        use base64::{Engine as _, engine::general_purpose};
        use regex::Regex;
        
        // 查找所有页面标记的位置
        let page_marker_regex = Regex::new(r"(<p>|Page )(\d+)(:|</p>)").unwrap();
        
        let mut result = String::new();
        let mut image_idx = 0;
        
        // 按段落分割 HTML，在段落之间插入图片
        let paragraph_regex = Regex::new(r"(</p>|</h[1-6]>|</li>|</ul>|</ol>|</div>)").unwrap();
        
        // 简单方法：在每 N 个段落或页面标记后插入一张图片
        let mut positions = Vec::new();
        
        // 查找所有可能插入图片的位置（段落结束、标题结束、列表结束等）
        for mat in paragraph_regex.find_iter(html_content) {
            positions.push(mat.end());
        }
        
        // 如果找到了页面标记，优先在页面标记后插入
        for mat in page_marker_regex.find_iter(html_content) {
            // 尝试从标记中提取页面号
            if let Some(_captures) = page_marker_regex.captures(&html_content[mat.start()..mat.end()]) {
                // 页面标记位置，可以在这里插入图片
                positions.push(mat.end());
            }
        }
        
        // 如果位置太少，在文档中间均匀分布
        if positions.is_empty() || positions.len() < images.len() {
            return self.insert_images_evenly(html_content, images);
        }
        
        // 计算每张图片应该插入的位置
        let images_per_position = if images.len() <= positions.len() {
            positions.len() / images.len().max(1)
        } else {
            1
        };
        
        let mut current_pos = 0;
        for (idx, &pos) in positions.iter().enumerate() {
            if idx > 0 && idx % images_per_position == 0 && image_idx < images.len() {
                // 插入图片
                let image = &images[image_idx];
                let base64_data = general_purpose::STANDARD.encode(&image.data);
                let mime_type = match image.format {
                    crate::import::converter::ImageFormat::Png => "image/png",
                    crate::import::converter::ImageFormat::Jpeg => "image/jpeg",
                    crate::import::converter::ImageFormat::Gif => "image/gif",
                    crate::import::converter::ImageFormat::Bmp => "image/bmp",
                    crate::import::converter::ImageFormat::WebP => "image/webp",
                };
                let data_url = format!("data:{};base64,{}", mime_type, base64_data);
                
                let (width, height) = image.dimensions.as_ref()
                    .map(|d| (d.width, d.height))
                    .unwrap_or((300, 200));
                
                let image_html = format!(
                    r#"<p><img src="{}" alt="{}" width="{}" height="{}" style="max-width: 100%; height: auto;" /></p>"#,
                    data_url, image.filename, width, height
                );
                
                result.push_str(&html_content[current_pos..pos]);
                result.push_str(&image_html);
                current_pos = pos;
                image_idx += 1;
            }
        }
        
        // 添加剩余内容
        result.push_str(&html_content[current_pos..]);
        
        // 如果还有未插入的图片，追加到末尾（在最后一个 </div> 之前）
        if image_idx < images.len() {
            let mut remaining_images = String::from("\n<div class=\"pdf-images\">\n");
            for image in &images[image_idx..] {
                let base64_data = general_purpose::STANDARD.encode(&image.data);
                let mime_type = match image.format {
                    crate::import::converter::ImageFormat::Png => "image/png",
                    crate::import::converter::ImageFormat::Jpeg => "image/jpeg",
                    crate::import::converter::ImageFormat::Gif => "image/gif",
                    crate::import::converter::ImageFormat::Bmp => "image/bmp",
                    crate::import::converter::ImageFormat::WebP => "image/webp",
                };
                let data_url = format!("data:{};base64,{}", mime_type, base64_data);
                
                let (width, height) = image.dimensions.as_ref()
                    .map(|d| (d.width, d.height))
                    .unwrap_or((300, 200));
                
                remaining_images.push_str(&format!(
                    r#"<p><img src="{}" alt="{}" width="{}" height="{}" style="max-width: 100%; height: auto;" /></p>"#,
                    data_url, image.filename, width, height
                ));
            }
            remaining_images.push_str("\n</div>\n");
            
            if let Some(pos) = result.rfind("</div>") {
                result.insert_str(pos, &remaining_images);
            } else {
                result.push_str(&remaining_images);
            }
        }
        
        result
    }
    
    /// 在整个文档中均匀分布图片
    fn insert_images_evenly(&self, html_content: &str, images: &[ExtractedImage]) -> String {
        use base64::{Engine as _, engine::general_purpose};
        use regex::Regex;
        
        // 查找所有段落结束位置
        let paragraph_end_regex = Regex::new(r"(</p>|</h[1-6]>|</blockquote>)").unwrap();
        
        let mut positions = Vec::new();
        for mat in paragraph_end_regex.find_iter(html_content) {
            positions.push(mat.end());
        }
        
        if positions.is_empty() {
            // 如果没有找到段落，直接追加到末尾
            return self.append_images_to_end(html_content, images);
        }
        
        // 计算每张图片应该插入的位置间隔
        let interval = if images.len() > 0 {
            (positions.len() / images.len().max(1)).max(1)
        } else {
            1
        };
        
        let mut result = String::new();
        let mut current_pos = 0;
        let mut image_idx = 0;
        
        for (idx, &pos) in positions.iter().enumerate() {
            result.push_str(&html_content[current_pos..pos]);
            
            // 每隔一定数量的段落插入一张图片
            if (idx + 1) % interval == 0 && image_idx < images.len() {
                let image = &images[image_idx];
                let base64_data = general_purpose::STANDARD.encode(&image.data);
                let mime_type = match image.format {
                    crate::import::converter::ImageFormat::Png => "image/png",
                    crate::import::converter::ImageFormat::Jpeg => "image/jpeg",
                    crate::import::converter::ImageFormat::Gif => "image/gif",
                    crate::import::converter::ImageFormat::Bmp => "image/bmp",
                    crate::import::converter::ImageFormat::WebP => "image/webp",
                };
                let data_url = format!("data:{};base64,{}", mime_type, base64_data);
                
                let (width, height) = image.dimensions.as_ref()
                    .map(|d| (d.width, d.height))
                    .unwrap_or((300, 200));
                
                let image_html = format!(
                    r#"<p><img src="{}" alt="{}" width="{}" height="{}" style="max-width: 100%; height: auto;" /></p>"#,
                    data_url, image.filename, width, height
                );
                
                result.push_str(&image_html);
                image_idx += 1;
            }
            
            current_pos = pos;
        }
        
        // 添加剩余内容
        result.push_str(&html_content[current_pos..]);
        
        // 如果还有未插入的图片，追加到末尾
        if image_idx < images.len() {
            let remaining_html = self.append_images_to_end("", &images[image_idx..]);
            if let Some(pos) = result.rfind("</div>") {
                result.insert_str(pos, &remaining_html);
            } else {
                result.push_str(&remaining_html);
            }
        }
        
        result
    }
    
    /// 将图片追加到 HTML 末尾（备用方法）
    fn append_images_to_end(&self, html_content: &str, images: &[ExtractedImage]) -> String {
        use base64::{Engine as _, engine::general_purpose};
        
        if images.is_empty() {
            return html_content.to_string();
        }
        
        let mut images_html = String::from("\n<div class=\"pdf-images\">\n");
        
        for image in images {
            let base64_data = general_purpose::STANDARD.encode(&image.data);
            let mime_type = match image.format {
                crate::import::converter::ImageFormat::Png => "image/png",
                crate::import::converter::ImageFormat::Jpeg => "image/jpeg",
                crate::import::converter::ImageFormat::Gif => "image/gif",
                crate::import::converter::ImageFormat::Bmp => "image/bmp",
                crate::import::converter::ImageFormat::WebP => "image/webp",
            };
            let data_url = format!("data:{};base64,{}", mime_type, base64_data);
            
            let (width, height) = image.dimensions.as_ref()
                .map(|d| (d.width, d.height))
                .unwrap_or((300, 200));
            
            images_html.push_str(&format!(
                r#"<p><img src="{}" alt="{}" width="{}" height="{}" style="max-width: 100%; height: auto;" /></p>"#,
                data_url, image.filename, width, height
            ));
        }
        images_html.push_str("\n</div>\n");
        
        let mut result = html_content.to_string();
        if result.contains("</div>") {
            if let Some(pos) = result.rfind("</div>") {
                result.insert_str(pos, &images_html);
            } else {
                result.push_str(&images_html);
            }
        } else {
            result.push_str(&images_html);
        }
        
        result
    }

    /// 提取页面中的图像
    fn extract_images(&self, pdf_data: &[u8]) -> FlowyResult<Vec<ExtractedImage>> {
        use uuid::Uuid;
        use crate::import::converter::{ImageFormat, ImageDimensions};
        
        // 创建临时PDF文件
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(pdf_data)?;
        temp_file.flush()?;

        // 查找 pdfimages 的完整路径
        let pdfimages_path = match find_tool_path("pdfimages") {
            Some(path) => path,
            None => {
                warn!("pdfimages command not found. Images will not be extracted.");
                return Ok(Vec::new());
            }
        };

        // 创建临时目录存储提取的图片
        let output_dir = tempfile::tempdir()?;
        let output_prefix = output_dir.path().join("img");

        info!("Extracting images from PDF using pdfimages (path: {})", pdfimages_path);

        // 执行 pdfimages 命令
        // -all: 提取所有类型的图像
        // 不指定 -png，使用原始格式
        let output = match Command::new(&pdfimages_path)
            .arg("-all")         // 提取所有图像类型
            .arg(temp_file.path())
            .arg(&output_prefix)
            .output()
        {
            Ok(output) => output,
            Err(e) => {
                warn!("pdfimages execution failed: {}", e);
                return Ok(Vec::new());
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("pdfimages failed: {}", stderr);
            return Ok(Vec::new());
        }

        // 查找生成的图片文件
        // pdfimages 会根据原图格式生成文件（可能是 .png, .jpg, .ppm 等）
        // pdfimages 的文件命名规则：如果输出前缀是 "img"，则生成 img-000.png, img-001.png 等
        info!("Checking output directory for images: {}", output_dir.path().display());
        
        // 列出输出目录中的所有文件，用于调试
        if let Ok(entries) = std::fs::read_dir(output_dir.path()) {
            let files: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            info!("Files in output directory: {:?}", files);
        }
        
        let mut extracted_images = Vec::new();
        let mut image_num = 0;

        // 检查常见的图片格式
        let image_extensions = vec!["png", "jpg", "jpeg", "ppm", "pbm"];
        
        loop {
            let mut found = false;
            
            for ext in &image_extensions {
                // pdfimages 使用 3 位数字：img-000.png, img-001.png, ...
                let image_file = output_dir.path().join(format!("img-{:03}.{}", image_num, ext));
                if image_file.exists() {
                    found = true;
                    
                    match std::fs::read(&image_file) {
                        Ok(image_data) if !image_data.is_empty() => {
                            // 检测图片格式
                            let format = if image_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                                ImageFormat::Png
                            } else if image_data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                                ImageFormat::Jpeg
                            } else if ext == &"ppm" || ext == &"pbm" {
                                // PPM/PBM 格式，需要转换为 PNG
                                match image::load_from_memory(&image_data) {
                                    Ok(img) => {
                                        use std::io::Cursor;
                                        let width = img.width();
                                        let height = img.height();
                                        let mut png_data = Vec::new();
                                        let mut cursor = Cursor::new(&mut png_data);
                                        if img.write_to(&mut cursor, image::ImageFormat::Png).is_ok() {
                                            let dimensions = Some(ImageDimensions {
                                                width,
                                                height,
                                            });
                                            
                                            let image_id = Uuid::new_v4().to_string();
                                            let filename = format!("image_{}.png", image_num);
                                            
                                            let extracted_image = ExtractedImage {
                                                id: image_id,
                                                filename,
                                                data: png_data,
                                                format: ImageFormat::Png,
                                                dimensions,
                                            };
                                            
                                            info!("Extracted and converted image: {} ({} bytes, {}x{})", 
                                                extracted_image.filename,
                                                extracted_image.data.len(),
                                                width,
                                                height);
                                            
                                            extracted_images.push(extracted_image);
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to convert PPM/PBM image: {}", e);
                                    }
                                }
                                break; // 已处理，跳出 ext 循环
                            } else {
                                ImageFormat::Png // 默认
                            };

                            // 尝试获取图片尺寸
                            let dimensions = image::load_from_memory(&image_data)
                                .ok()
                                .map(|img| ImageDimensions {
                                    width: img.width(),
                                    height: img.height(),
                                });

                            // 在移动 dimensions 之前获取尺寸值用于日志
                            let (width, height) = dimensions.as_ref()
                                .map(|d| (d.width, d.height))
                                .unwrap_or((0, 0));

                            let image_id = Uuid::new_v4().to_string();
                            let extension = match format {
                                ImageFormat::Png => "png",
                                ImageFormat::Jpeg => "jpg",
                                ImageFormat::Gif => "gif",
                                ImageFormat::Bmp => "bmp",
                                ImageFormat::WebP => "webp",
                            };
                            let filename = format!("image_{}.{}", image_num, extension);

                            let extracted_image = ExtractedImage {
                                id: image_id,
                                filename,
                                data: image_data,
                                format,
                                dimensions,
                            };

                            info!("Extracted image: {} ({} bytes, {}x{})", 
                                extracted_image.filename,
                                extracted_image.data.len(),
                                width,
                                height);
                            
                            extracted_images.push(extracted_image);
                            break; // 已找到并处理，跳出 ext 循环
                        }
                        Ok(_) => {
                            warn!("Empty image file: {}", image_file.display());
                            break;
                        }
                        Err(e) => {
                            warn!("Failed to read image file {}: {}", image_file.display(), e);
                            break;
                        }
                    }
                }
            }
            
            if !found {
                // 没有找到更多图片文件，退出循环
                break;
            }
            
            image_num += 1;
            
            // 限制最大图片数量，避免无限循环
            if image_num > 1000 {
                warn!("Reached maximum image count limit (1000). Stopping image extraction.");
                break;
            }
        }

        info!("Extracted {} images from PDF", extracted_images.len());
        if extracted_images.is_empty() {
            warn!("No images were extracted from PDF. This may be normal if the PDF contains no images, or pdfimages may have failed.");
        }
        Ok(extracted_images)
    }
}

impl Drop for NativePdfConverter {
    fn drop(&mut self) {
        // 系统工具不需要特殊清理
    }
}

#[async_trait::async_trait]
impl DocumentConverter for NativePdfConverter {
    fn name(&self) -> &str {
        "System Tools PDF Converter"
    }

    fn supported_types(&self) -> Vec<DocumentType> {
        vec![DocumentType::Pdf]
    }

    async fn convert(&self, task: &ConversionTask) -> FlowyResult<ConversionResult> {
        let start_time = std::time::Instant::now();
        
        info!("[PDF导入进度] 开始处理文件: {}", task.source_path);

        // 读取PDF文件数据
        let pdf_data = if let Some(bytes) = &task.bytes_data {
            bytes.clone()
        } else {
            // 验证文件
            self.validate_file(Path::new(&task.source_path)).await?;
            
            // 读取PDF文件
            let mut file = File::open(&task.source_path)
                .map_err(|e| FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("Failed to open PDF file: {}", e),
                ))?;

            let mut pdf_data = Vec::new();
            file.read_to_end(&mut pdf_data)
                .map_err(|e| FlowyError::new(
                    flowy_error::ErrorCode::Internal,
                    format!("Failed to read PDF file: {}", e),
                ))?;
            
            pdf_data
        };

        info!("[PDF导入进度] 文件已读取: {} 字节 ({:.2} MB)", 
            pdf_data.len(), pdf_data.len() as f64 / (1024.0 * 1024.0));

        // 创建转换器实例来调用extract_text_with_system_tools
        let converter = NativePdfConverter::new(self.config.clone());
        
        // 提取文档元数据
        let metadata = converter.extract_document_metadata(&pdf_data, &task.source_path).await?;

        // 先提取图像（使用 pdfimages 工具），以便在HTML中插入占位符
        info!("[PDF导入进度] 正在提取图片...");
        let extracted_images = match converter.extract_images(&pdf_data) {
            Ok(images) => {
                info!("[PDF导入进度] 图片提取完成: 找到 {} 张图片", images.len());
                images
            }
            Err(e) => {
                warn!("Failed to extract images using pdfimages: {}", e);
                Vec::new()
            }
        };
        
        // 提取文本内容
        let extracted_text = converter.extract_text_with_system_tools(&pdf_data)?;
        
        // 直接使用提取的文本内容，不做任何 HTML 转换
        let document_content = extracted_text;
        let word_count = document_content.split_whitespace().count();
        let text_length = document_content.len();
        
        // 统计表格（以 | 开头且包含多个 | 的行）
        let table_count = document_content.lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with('|') && trimmed.matches('|').count() >= 3
            })
            .count() as u32;

        let processing_time = start_time.elapsed().as_millis() as u64;
        let file_size = task.bytes_data.as_ref().map(|bytes| bytes.len() as u64).unwrap_or(0);
        let image_count = extracted_images.len() as u32;

        info!("Native PDFium conversion completed in {}ms", processing_time);
        info!("Extracted {} characters, {} words, {} images", text_length, word_count, extracted_images.len());

        let result = ConversionResult {
            task_id: task.id,
            document_content: DocumentContent {
                title: task.target_name.clone(),
                content: document_content,
                metadata: DocumentMetadata {
                    author: metadata.author,
                    created_at: metadata.created_at,
                    modified_at: metadata.modified_at,
                    page_count: metadata.page_count,
                    word_count: Some(word_count as u32),
                    language: metadata.language,
                },
            },
            extracted_images,
            statistics: ConversionStatistics {
                processing_time_ms: processing_time,
                text_length,
                image_count,
                table_count,
                source_file_size: file_size,
            },
        };

        Ok(result)
    }

    async fn validate_file(&self, file_path: &Path) -> FlowyResult<()> {
        // 检查文件是否存在
        if !file_path.exists() {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::RecordNotFound,
                format!("File not found: {}", file_path.display()),
            ));
        }

        // 检查文件扩展名
        if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
            if !DocumentType::Pdf.supported_extensions().contains(&ext.to_lowercase().as_str()) {
                return Err(FlowyError::new(
                    flowy_error::ErrorCode::UnsupportedFileFormat,
                    format!("Unsupported file format: {}", ext),
                ));
            }
        } else {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::UnsupportedFileFormat,
                "File has no extension",
            ));
        }

        // 检查文件大小
        if let Some(max_size) = self.config.max_file_size {
            let file_size = std::fs::metadata(file_path)?.len();
            if file_size > max_size {
                return Err(FlowyError::new(
                    flowy_error::ErrorCode::SingleUploadLimitExceeded,
                    format!("File too large: {} bytes (max: {} bytes)", file_size, max_size),
                ));
            }
        }

        // 检查PDF文件头
        let mut file = std::fs::File::open(file_path)?;
        let mut header = [0u8; 4];
        std::io::Read::read_exact(&mut file, &mut header)?;
        
        if &header != b"%PDF" {
            return Err(FlowyError::new(
                flowy_error::ErrorCode::UnsupportedFileFormat,
                "Invalid PDF file format",
            ));
        }

        Ok(())
    }

    async fn get_file_info(&self, file_path: &Path) -> FlowyResult<DocumentMetadata> {
        // 验证文件
        self.validate_file(file_path).await?;

        // 读取PDF文件
        let mut file = File::open(file_path)
            .map_err(|e| FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("Failed to open PDF file: {}", e),
            ))?;

        let mut pdf_data = Vec::new();
        file.read_to_end(&mut pdf_data)
            .map_err(|e| FlowyError::new(
                flowy_error::ErrorCode::Internal,
                format!("Failed to read PDF file: {}", e),
            ))?;

        // 创建转换器实例来调用extract_document_metadata
        let converter = NativePdfConverter::new(self.config.clone());
        
        // 提取元数据
        converter.extract_document_metadata(&pdf_data, &file_path.to_string_lossy()).await
    }
}
