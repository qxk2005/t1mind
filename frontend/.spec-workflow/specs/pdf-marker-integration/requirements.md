# PDF Marker 集成需求规范

## 介绍

本文档定义了重新设计 PDF 转换逻辑的需求，将现有的 `pdftohtml` -> `htmltomarkdown` -> AppFlowy 文档的转换流程，替换为使用 Marker 工具直接将 PDF 转换为 Markdown，然后转换为 AppFlowy 文档。该改进将显著提升 PDF 转换的准确性和性能，同时简化转换流程和维护成本。

Marker 是一个开源工具，能够快速、准确地将 PDF 转换为 Markdown。它支持多种文档类型（包括书籍和科学论文），支持多语言，并能移除页眉、页脚等非内容元素，格式化表格和代码块，提取并保存图像，以及将大多数方程式转换为 LaTeX 格式。

**重要要求**：Marker 工具应集成到 macOS 和 Windows 应用中独立分发，用户无需自行安装 Marker 工具。这意味着 Marker 工具应作为二进制文件打包到应用包中，并在运行时从应用包内加载。

## 与产品愿景的契合

该功能直接支持 AppFlowy 作为统一生产力平台的愿景，通过提供更准确、更高效的 PDF 导入能力，提升用户体验。同时，该功能体现了 AppFlowy 对技术创新和代码质量的承诺，通过使用更先进的工具来简化系统架构。

## 需求

### 需求 1：集成 Marker 工具并打包到应用中

**用户故事：** 作为一名开发者，我希望在 Rust 后端集成 Marker 工具，并将其打包到 macOS 和 Windows 应用中，以便用户无需自行安装即可使用。

#### 验收标准

1. WHEN 系统需要转换 PDF 文档时 THEN 系统 SHALL 使用 Marker 工具而不是 pdftohtml
2. WHEN 打包应用时 THEN 系统 SHALL 将 Marker 工具二进制文件包含在应用包中
3. WHEN 运行时查找 Marker 工具时 THEN 系统 SHALL 优先从应用包内查找 Marker 工具，而不是从系统 PATH
4. WHEN 调用 Marker 工具时 THEN 系统 SHALL 通过 Rust 子进程管理库（如 `std::process`）执行 Marker 命令
5. WHEN Marker 工具执行失败时 THEN 系统 SHALL 捕获错误信息并记录到日志
6. WHEN 成功执行 Marker 工具时 THEN 系统 SHALL 获取生成的 Markdown 输出
7. IF Marker 工具在应用包中不可用时 THEN 系统 SHALL 记录错误并尝试回退方案（如果可用）

### 需求 2：移除 HTML 转换中间步骤

**用户故事：** 作为一名开发者，我希望移除 HTML 转换的中间步骤，以便简化转换流程并减少潜在的转换错误。

#### 验收标准

1. WHEN 重构 PDF 转换逻辑时 THEN 系统 SHALL 移除所有 pdftohtml 相关的代码
2. WHEN 重构 PDF 转换逻辑时 THEN 系统 SHALL 移除所有 htmltomarkdown 相关的代码
3. WHEN 重构完成后 THEN 系统 SHALL 不再依赖 poppler-utils 工具（pdftohtml）
4. WHEN 重构完成后 THEN 系统 SHALL 不再依赖 html2md 库
5. WHEN 转换流程执行时 THEN 系统 SHALL 直接从 PDF -> Markdown -> AppFlowy 文档

### 需求 3：Markdown 到 AppFlowy 文档转换

**用户故事：** 作为一名用户，我希望 Marker 生成的 Markdown 能够正确转换为 AppFlowy 文档格式，以便保留原始文档的结构和内容。

#### 验收标准

1. WHEN 系统接收到 Marker 生成的 Markdown 时 THEN 系统 SHALL 解析 Markdown 内容
2. WHEN 解析 Markdown 时 THEN 系统 SHALL 保留标题层级结构（H1-H6）
3. WHEN 解析 Markdown 时 THEN 系统 SHALL 保留表格结构和内容
4. WHEN 解析 Markdown 时 THEN 系统 SHALL 保留代码块和代码片段
5. WHEN 解析 Markdown 时 THEN 系统 SHALL 保留列表（有序和无序）
6. WHEN 解析 Markdown 时 THEN 系统 SHALL 保留图片引用并处理图片数据
7. WHEN 解析 Markdown 时 THEN 系统 SHALL 处理 LaTeX 格式的数学公式（如果存在）
8. WHEN 转换完成时 THEN 系统 SHALL 生成符合 AppFlowy 文档格式的内容

### 需求 4：图片提取和处理

**用户故事：** 作为一名用户，我希望 PDF 中的图片能够被正确提取并嵌入到 AppFlowy 文档中。

#### 验收标准

1. WHEN Marker 提取 PDF 中的图片时 THEN 系统 SHALL 保留图片文件或图片数据
2. WHEN 处理 Markdown 中的图片引用时 THEN 系统 SHALL 将图片嵌入到 AppFlowy 文档中
3. IF 图片提取失败 THEN 系统 SHALL 记录警告但继续处理其他内容
4. WHEN 图片过大时 THEN 系统 SHALL 进行适当的压缩或尺寸调整
5. WHEN 处理图片时 THEN 系统 SHALL 支持常见的图片格式（PNG、JPEG、GIF）

### 需求 5：Marker 工具打包和分发

**用户故事：** 作为一名开发者，我希望 Marker 工具能够自动打包到 macOS 和 Windows 应用包中，以便用户无需额外安装步骤。

#### 验收标准

1. WHEN 构建 macOS 应用时 THEN 系统 SHALL 将 Marker 工具二进制文件（或 Python 环境）打包到 .app 包中
2. WHEN 构建 Windows 应用时 THEN 系统 SHALL 将 Marker 工具二进制文件（或 Python 环境）打包到安装包中
3. WHEN 应用启动时 THEN 系统 SHALL 验证 Marker 工具是否在应用包中可用
4. WHEN 运行时调用 Marker 工具时 THEN 系统 SHALL 使用应用包内的 Marker 工具路径
5. IF Marker 工具打包失败 THEN 系统 SHALL 在构建时提供清晰的错误提示
6. WHEN 打包 Marker 工具时 THEN 系统 SHALL 处理所有必要的依赖项（如 Python 解释器、Python 包等，如果 Marker 基于 Python）

### 需求 6：错误处理和回退机制

**用户故事：** 作为一名用户，我希望即使 Marker 工具失败，系统也能够提供备用的转换方案，以便不中断我的工作流程。

#### 验收标准

1. IF Marker 工具执行失败 THEN 系统 SHALL 记录详细的错误信息
2. IF Marker 工具在应用包中不可用时 THEN 系统 SHALL 提供清晰的错误提示，说明内部组件缺失
3. IF 转换过程中出现错误 THEN 系统 SHALL 提供部分转换结果（如果可能）
4. WHEN 错误发生时 THEN 系统 SHALL 记录完整的错误堆栈和上下文信息
5. IF 用户请求时 THEN 系统 SHALL 提供错误日志和调试信息

### 需求 7：性能优化

**用户故事：** 作为一名用户，我希望 PDF 转换过程能够快速完成，以便不耽误我的工作。

#### 验收标准

1. WHEN 转换小型 PDF 文档（< 10MB）时 THEN 系统 SHALL 在 30 秒内完成转换
2. WHEN 转换中型 PDF 文档（10-50MB）时 THEN 系统 SHALL 在 2 分钟内完成转换
3. WHEN 转换大型 PDF 文档（> 50MB）时 THEN 系统 SHALL 提供进度指示
4. WHEN 转换过程中 THEN 系统 SHALL 合理使用系统资源，避免内存溢出
5. WHEN 批量转换多个 PDF 时 THEN 系统 SHALL 支持并发处理（在合理范围内）

### 需求 8：代码清理和维护

**用户故事：** 作为一名开发者，我希望代码库中移除所有不再使用的代码，以便降低维护成本和提高代码可读性。

#### 验收标准

1. WHEN 重构完成后 THEN 系统 SHALL 移除 `pdf_converter_native.rs` 中所有 pdftohtml 相关代码
2. WHEN 重构完成后 THEN 系统 SHALL 移除所有 html2md 转换相关代码
3. WHEN 重构完成后 THEN 系统 SHALL 更新 Cargo.toml，移除不再需要的依赖
4. WHEN 重构完成后 THEN 系统 SHALL 更新所有相关的文档和注释
5. WHEN 重构完成后 THEN 系统 SHALL 确保所有测试用例通过

## 非功能性需求

### 代码架构和模块化
- **单一职责原则**：PDF 转换器应专注于 PDF 到 Markdown 的转换，Markdown 到 AppFlowy 的转换应在单独的模块中
- **模块化设计**：Marker 工具调用应封装在独立的模块中，便于测试和维护
- **依赖管理**：最小化外部工具依赖，优先使用 Rust 原生解决方案
- **清晰接口**：定义清晰的转换接口，支持未来可能的其他转换工具

### 性能
- Marker 工具执行时间应计入总转换时间
- 支持处理最大 100MB 的单个 PDF 文件
- 内存使用应控制在合理范围内，避免内存泄漏
- 转换进度更新频率不超过每秒 2 次

### 安全性
- 所有文件操作应在沙盒环境中进行
- Marker 工具执行应使用安全的子进程调用
- 验证 Marker 工具的输出，防止恶意内容注入
- 临时文件应在使用后立即清理

### 可靠性
- 转换成功率应达到 95% 以上（与现有实现相当或更好）
- 系统应能够从 Marker 工具失败中恢复
- 提供详细的错误日志和诊断信息
- 支持转换超时机制

### 可用性
- 用户界面应保持与现有导入功能一致
- 提供清晰的错误提示和安装指引（如果 Marker 未安装）
- 转换过程应提供进度反馈
- 错误信息应易于理解，提供解决方案建议

### 兼容性
- 支持 macOS、Linux 和 Windows 平台
- Marker 工具应作为应用包的一部分打包到 macOS 和 Windows 应用中
- 对于 Linux 平台，可考虑提供系统级安装指引或应用内打包方案
- 保持与现有 PDF 导入功能的接口兼容性
- 支持各种 PDF 格式（标准 PDF、扫描 PDF 等）
- 确保 Marker 工具及其依赖项（如 Python 环境）能够正确打包到各平台应用包中


