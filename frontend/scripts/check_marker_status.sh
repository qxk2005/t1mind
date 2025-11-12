#!/bin/bash
# Marker 和 marker-pdf 组件检查脚本
# 用于检查 marker 工具、marker-pdf 安装状态以及模型下载情况
# 方便在不同 macOS 机器上比对配置是否正确

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 输出文件
OUTPUT_FILE="${1:-marker_status_$(date +%Y%m%d_%H%M%S).txt}"

# 报告函数
report() {
    local status=$1
    local message=$2
    case $status in
        "OK")
            echo -e "${GREEN}✓${NC} $message" | tee -a "$OUTPUT_FILE"
            ;;
        "WARN")
            echo -e "${YELLOW}⚠${NC} $message" | tee -a "$OUTPUT_FILE"
            ;;
        "ERROR")
            echo -e "${RED}✗${NC} $message" | tee -a "$OUTPUT_FILE"
            ;;
        "INFO")
            echo -e "${BLUE}ℹ${NC} $message" | tee -a "$OUTPUT_FILE"
            ;;
        "SECTION")
            echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}" | tee -a "$OUTPUT_FILE"
            echo -e "${CYAN}$message${NC}" | tee -a "$OUTPUT_FILE"
            echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}" | tee -a "$OUTPUT_FILE"
            ;;
        *)
            echo "$message" | tee -a "$OUTPUT_FILE"
            ;;
    esac
}

# 计算目录大小
get_dir_size() {
    local dir=$1
    if [ -d "$dir" ]; then
        du -sh "$dir" 2>/dev/null | cut -f1
    else
        echo "0"
    fi
}

# 计算文件数量
get_file_count() {
    local dir=$1
    if [ -d "$dir" ]; then
        find "$dir" -type f 2>/dev/null | wc -l | tr -d ' '
    else
        echo "0"
    fi
}

# 获取文件哈希（用于比对）
get_file_hash() {
    local file=$1
    if [ -f "$file" ]; then
        if command -v shasum >/dev/null 2>&1; then
            shasum -a 256 "$file" 2>/dev/null | cut -d' ' -f1
        elif command -v sha256sum >/dev/null 2>&1; then
            sha256sum "$file" 2>/dev/null | cut -d' ' -f1
        else
            echo "N/A (no hash tool)"
        fi
    else
        echo "N/A (file not found)"
    fi
}

# 开始检查
echo "" | tee "$OUTPUT_FILE"
report "SECTION" "Marker 和 marker-pdf 组件检查报告"
echo "生成时间: $(date)" | tee -a "$OUTPUT_FILE"
echo "主机名: $(hostname)" | tee -a "$OUTPUT_FILE"
echo "用户: $(whoami)" | tee -a "$OUTPUT_FILE"
echo "系统: $(uname -a)" | tee -a "$OUTPUT_FILE"
echo "" | tee -a "$OUTPUT_FILE"

# ==========================================
# 1. 检查 marker 脚本
# ==========================================
report "SECTION" "1. Marker 脚本检查"

# 查找 marker 脚本的可能位置
MARKER_PATHS=(
    "$HOME/Documents/Program/t1mind/frontend/resources/marker/marker"
    "./resources/marker/marker"
    "../resources/marker/marker"
    "../../resources/marker/marker"
    "/Applications/AppFlowy.app/Contents/Resources/marker/marker"
)

MARKER_FOUND=false
MARKER_PATH=""

for path in "${MARKER_PATHS[@]}"; do
    if [ -f "$path" ]; then
        MARKER_PATH="$path"
        MARKER_FOUND=true
        break
    fi
done

# 也尝试从当前工作目录查找
if [ "$MARKER_FOUND" = false ]; then
    CURRENT_DIR=$(pwd)
    POSSIBLE_MARKER="$CURRENT_DIR/resources/marker/marker"
    if [ -f "$POSSIBLE_MARKER" ]; then
        MARKER_PATH="$POSSIBLE_MARKER"
        MARKER_FOUND=true
    fi
fi

if [ "$MARKER_FOUND" = true ]; then
    report "OK" "Marker 脚本找到: $MARKER_PATH"
    echo "  绝对路径: $(cd "$(dirname "$MARKER_PATH")" && pwd)/$(basename "$MARKER_PATH")" | tee -a "$OUTPUT_FILE"
    
    # 检查文件权限
    if [ -x "$MARKER_PATH" ]; then
        report "OK" "Marker 脚本可执行"
    else
        report "WARN" "Marker 脚本不可执行，尝试添加执行权限..."
        chmod +x "$MARKER_PATH" 2>/dev/null || true
    fi
    
    # 文件信息
    FILE_SIZE=$(ls -lh "$MARKER_PATH" | awk '{print $5}')
    FILE_HASH=$(get_file_hash "$MARKER_PATH")
    echo "  文件大小: $FILE_SIZE" | tee -a "$OUTPUT_FILE"
    echo "  文件哈希 (SHA256): $FILE_HASH" | tee -a "$OUTPUT_FILE"
    
    # 检查脚本内容（前几行）
    echo "  脚本前 10 行:" | tee -a "$OUTPUT_FILE"
    head -10 "$MARKER_PATH" | sed 's/^/    /' | tee -a "$OUTPUT_FILE"
else
    report "ERROR" "Marker 脚本未找到"
    echo "  已检查的位置:" | tee -a "$OUTPUT_FILE"
    for path in "${MARKER_PATHS[@]}"; do
        echo "    - $path" | tee -a "$OUTPUT_FILE"
    done
fi

echo "" | tee -a "$OUTPUT_FILE"

# ==========================================
# 2. 检查 marker-pdf (pipx 安装)
# ==========================================
report "SECTION" "2. marker-pdf 安装检查"

# 检查 pipx
if command -v pipx >/dev/null 2>&1; then
    PIPX_VERSION=$(pipx --version 2>&1 || echo "unknown")
    report "OK" "pipx 已安装: $PIPX_VERSION"
    echo "  pipx 路径: $(which pipx)" | tee -a "$OUTPUT_FILE"
    
    # 检查 pipx 安装的包
    echo "  pipx 已安装的包:" | tee -a "$OUTPUT_FILE"
    pipx list 2>/dev/null | sed 's/^/    /' | tee -a "$OUTPUT_FILE" || echo "    (无法获取列表)" | tee -a "$OUTPUT_FILE"
else
    report "ERROR" "pipx 未安装"
    echo "  安装方法: brew install pipx" | tee -a "$OUTPUT_FILE"
fi

echo "" | tee -a "$OUTPUT_FILE"

# 检查 marker-pdf
MARKER_PDF_PATHS=(
    "$HOME/.local/pipx/venvs/marker-pdf/bin/marker_single"
    "$HOME/.local/pipx/venvs/marker-pdf/bin/python"
)

MARKER_PDF_FOUND=false
MARKER_PDF_SCRIPT=""
MARKER_PDF_PYTHON=""

for path in "${MARKER_PDF_PATHS[@]}"; do
    if [ -f "$path" ]; then
        if [[ "$path" == *"marker_single"* ]]; then
            MARKER_PDF_SCRIPT="$path"
            MARKER_PDF_FOUND=true
        elif [[ "$path" == *"python"* ]]; then
            MARKER_PDF_PYTHON="$path"
        fi
    fi
done

if [ "$MARKER_PDF_FOUND" = true ] && [ -n "$MARKER_PDF_SCRIPT" ]; then
    report "OK" "marker-pdf 已安装"
    echo "  marker_single 路径: $MARKER_PDF_SCRIPT" | tee -a "$OUTPUT_FILE"
    echo "  Python 路径: ${MARKER_PDF_PYTHON:-N/A}" | tee -a "$OUTPUT_FILE"
    
    # 检查 Python 版本
    if [ -n "$MARKER_PDF_PYTHON" ] && [ -f "$MARKER_PDF_PYTHON" ]; then
        PYTHON_VERSION=$("$MARKER_PDF_PYTHON" --version 2>&1 || echo "unknown")
        echo "  Python 版本: $PYTHON_VERSION" | tee -a "$OUTPUT_FILE"
    fi
    
    # 检查 marker-pdf 版本（如果支持）
    if [ -f "$MARKER_PDF_SCRIPT" ]; then
        echo "  尝试获取 marker-pdf 版本信息..." | tee -a "$OUTPUT_FILE"
        # 不实际执行，只检查文件
        FILE_SIZE=$(ls -lh "$MARKER_PDF_SCRIPT" | awk '{print $5}')
        echo "  marker_single 文件大小: $FILE_SIZE" | tee -a "$OUTPUT_FILE"
    fi
    
    # 检查 pipx venv 目录
    PIPX_VENV_DIR="$HOME/.local/pipx/venvs/marker-pdf"
    if [ -d "$PIPX_VENV_DIR" ]; then
        VENV_SIZE=$(get_dir_size "$PIPX_VENV_DIR")
        echo "  pipx venv 目录大小: $VENV_SIZE" | tee -a "$OUTPUT_FILE"
    fi
else
    report "ERROR" "marker-pdf 未安装"
    echo "  期望位置: $HOME/.local/pipx/venvs/marker-pdf/bin/marker_single" | tee -a "$OUTPUT_FILE"
    echo "  安装方法:" | tee -a "$OUTPUT_FILE"
    echo "    1. brew install jpeg libpng freetype openjpeg libtiff webp" | tee -a "$OUTPUT_FILE"
    echo "    2. brew install pipx" | tee -a "$OUTPUT_FILE"
    echo "    3. pipx install marker-pdf" | tee -a "$OUTPUT_FILE"
fi

echo "" | tee -a "$OUTPUT_FILE"

# ==========================================
# 3. 检查 Python 环境
# ==========================================
report "SECTION" "3. Python 环境检查"

# 系统 Python
if command -v python3 >/dev/null 2>&1; then
    SYS_PYTHON_VERSION=$(python3 --version 2>&1 || echo "unknown")
    report "OK" "系统 Python3: $SYS_PYTHON_VERSION"
    echo "  路径: $(which python3)" | tee -a "$OUTPUT_FILE"
else
    report "WARN" "系统 Python3 未找到"
fi

# pipx Python
if [ -n "$MARKER_PDF_PYTHON" ] && [ -f "$MARKER_PDF_PYTHON" ]; then
    PIPX_PYTHON_VERSION=$("$MARKER_PDF_PYTHON" --version 2>&1 || echo "unknown")
    report "OK" "pipx Python: $PIPX_PYTHON_VERSION"
    echo "  路径: $MARKER_PDF_PYTHON" | tee -a "$OUTPUT_FILE"
fi

echo "" | tee -a "$OUTPUT_FILE"

# ==========================================
# 4. 检查模型缓存目录
# ==========================================
report "SECTION" "4. 模型缓存目录检查"

# Hugging Face 缓存目录
HF_CACHE_DIR="$HOME/Library/Caches/huggingface"
HF_HUB_DIR="$HF_CACHE_DIR/hub"

if [ -d "$HF_CACHE_DIR" ]; then
    report "OK" "Hugging Face 缓存目录存在: $HF_CACHE_DIR"
    HF_SIZE=$(get_dir_size "$HF_CACHE_DIR")
    HF_FILES=$(get_file_count "$HF_CACHE_DIR")
    echo "  目录大小: $HF_SIZE" | tee -a "$OUTPUT_FILE"
    echo "  文件数量: $HF_FILES" | tee -a "$OUTPUT_FILE"
    
    # 列出主要模型目录
    if [ -d "$HF_HUB_DIR" ]; then
        echo "  Hub 目录内容:" | tee -a "$OUTPUT_FILE"
        ls -1 "$HF_HUB_DIR" 2>/dev/null | head -20 | sed 's/^/    - /' | tee -a "$OUTPUT_FILE" || echo "    (无法列出)" | tee -a "$OUTPUT_FILE"
        
        # 列出模型文件（查找常见的模型文件）
        echo "  模型文件详情:" | tee -a "$OUTPUT_FILE"
        find "$HF_HUB_DIR" -type f -name "*.bin" -o -name "*.safetensors" -o -name "*.pt" -o -name "*.pth" 2>/dev/null | head -10 | while read -r file; do
            FILE_SIZE=$(ls -lh "$file" | awk '{print $5}')
            echo "    - $(basename "$file"): $FILE_SIZE" | tee -a "$OUTPUT_FILE"
        done
    else
        report "WARN" "Hub 目录不存在: $HF_HUB_DIR"
    fi
else
    report "WARN" "Hugging Face 缓存目录不存在: $HF_CACHE_DIR"
    echo "  将在首次运行时自动创建" | tee -a "$OUTPUT_FILE"
fi

echo "" | tee -a "$OUTPUT_FILE"

# Surya OCR 模型缓存目录
SURYA_CACHE_DIR="$HOME/Library/Caches/datalab/models"

if [ -d "$SURYA_CACHE_DIR" ]; then
    report "OK" "Surya OCR 模型缓存目录存在: $SURYA_CACHE_DIR"
    SURYA_SIZE=$(get_dir_size "$SURYA_CACHE_DIR")
    SURYA_FILES=$(get_file_count "$SURYA_CACHE_DIR")
    echo "  目录大小: $SURYA_SIZE" | tee -a "$OUTPUT_FILE"
    echo "  文件数量: $SURYA_FILES" | tee -a "$OUTPUT_FILE"
    
    # 列出模型文件
    echo "  模型文件列表:" | tee -a "$OUTPUT_FILE"
    find "$SURYA_CACHE_DIR" -type f 2>/dev/null | while read -r file; do
        FILE_SIZE=$(ls -lh "$file" | awk '{print $5}')
        FILE_NAME=$(basename "$file")
        echo "    - $FILE_NAME: $FILE_SIZE" | tee -a "$OUTPUT_FILE"
    done
    
    if [ "$SURYA_FILES" -eq 0 ]; then
        report "WARN" "Surya OCR 模型目录为空，首次运行需要下载模型"
    fi
else
    report "WARN" "Surya OCR 模型缓存目录不存在: $SURYA_CACHE_DIR"
    echo "  将在首次运行时自动创建" | tee -a "$OUTPUT_FILE"
fi

echo "" | tee -a "$OUTPUT_FILE"

# ==========================================
# 5. 检查环境变量
# ==========================================
report "SECTION" "5. 环境变量检查"

# 检查 marker 脚本中设置的环境变量
ENV_VARS=(
    "HF_HOME"
    "HF_HUB_CACHE"
    "HUGGINGFACE_HUB_CACHE"
    "TRANSFORMERS_CACHE"
    "HF_DATASETS_CACHE"
    "HF_DATASETS_STORAGE_PATH"
    "HF_CACHE_DIR"
    "HF_HUB_CACHE_DIR"
    "SURYA_MODEL_CACHE_DIR"
    "PYTORCH_ENABLE_MPS_FALLBACK"
    "PYTORCH_MPS_HIGH_WATERMARK_RATIO"
    "PYTORCH_MPS_FORCE_CPU"
    "PYTORCH_MPS_DISABLE"
    "TORCH_DEVICE"
)

echo "当前环境变量设置:" | tee -a "$OUTPUT_FILE"
for var in "${ENV_VARS[@]}"; do
    value=$(eval "echo \$$var")
    if [ -n "$value" ]; then
        echo "  $var=$value" | tee -a "$OUTPUT_FILE"
    else
        echo "  $var=(未设置)" | tee -a "$OUTPUT_FILE"
    fi
done

echo "" | tee -a "$OUTPUT_FILE"

# 检查 marker 脚本中的环境变量设置
if [ -n "$MARKER_PATH" ] && [ -f "$MARKER_PATH" ]; then
    echo "marker 脚本中的环境变量设置:" | tee -a "$OUTPUT_FILE"
    grep -E "^(export |set )" "$MARKER_PATH" | grep -E "(HF_|SURYA_|PYTORCH_|TORCH_)" | sed 's/^/    /' | tee -a "$OUTPUT_FILE" || echo "    (未找到相关设置)" | tee -a "$OUTPUT_FILE"
fi

echo "" | tee -a "$OUTPUT_FILE"

# ==========================================
# 6. 检查依赖库
# ==========================================
report "SECTION" "6. 系统依赖检查"

# 检查 Homebrew
if command -v brew >/dev/null 2>&1; then
    BREW_VERSION=$(brew --version 2>&1 | head -1 || echo "unknown")
    report "OK" "Homebrew 已安装: $BREW_VERSION"
    
    # 检查 marker-pdf 需要的依赖
    DEPS=("jpeg" "libpng" "freetype" "openjpeg" "libtiff" "webp")
    echo "  检查 Pillow 编译依赖:" | tee -a "$OUTPUT_FILE"
    for dep in "${DEPS[@]}"; do
        if brew list "$dep" >/dev/null 2>&1; then
            DEP_VERSION=$(brew info "$dep" 2>/dev/null | grep "^$dep:" | head -1 || echo "installed")
            echo "    ✓ $dep: $DEP_VERSION" | tee -a "$OUTPUT_FILE"
        else
            echo "    ✗ $dep: 未安装" | tee -a "$OUTPUT_FILE"
        fi
    done
else
    report "WARN" "Homebrew 未安装"
    echo "  marker-pdf 需要 Homebrew 来安装依赖库" | tee -a "$OUTPUT_FILE"
fi

echo "" | tee -a "$OUTPUT_FILE"

# ==========================================
# 7. 测试 marker 脚本执行
# ==========================================
report "SECTION" "7. Marker 脚本执行测试"

if [ -n "$MARKER_PATH" ] && [ -f "$MARKER_PATH" ] && [ -x "$MARKER_PATH" ]; then
    # 尝试获取帮助信息（不实际执行转换）
    echo "尝试执行 marker --help..." | tee -a "$OUTPUT_FILE"
    
    # 检查是否有 timeout 命令
    if command -v timeout >/dev/null 2>&1; then
        # 使用 timeout 命令（超时 10 秒）
        if timeout 10 "$MARKER_PATH" --help >/dev/null 2>&1; then
            report "OK" "Marker 脚本可以正常执行"
        else
            EXIT_CODE=$?
            if [ $EXIT_CODE -eq 124 ]; then
                report "WARN" "Marker 脚本执行超时（可能卡住）"
            else
                report "WARN" "Marker 脚本执行失败（退出码: $EXIT_CODE）"
            fi
        fi
    else
        # 没有 timeout 命令，直接执行（可能卡住）
        report "INFO" "timeout 命令未找到，直接执行测试（可能卡住）"
        if "$MARKER_PATH" --help >/dev/null 2>&1; then
            report "OK" "Marker 脚本可以正常执行"
        else
            EXIT_CODE=$?
            report "WARN" "Marker 脚本执行失败（退出码: $EXIT_CODE）"
        fi
    fi
else
    report "WARN" "无法测试 Marker 脚本执行（脚本未找到或不可执行）"
fi

echo "" | tee -a "$OUTPUT_FILE"

# ==========================================
# 8. 生成摘要
# ==========================================
report "SECTION" "8. 检查摘要"

SUMMARY_OK=0
SUMMARY_WARN=0
SUMMARY_ERROR=0

# 统计
if [ "$MARKER_FOUND" = true ]; then
    ((SUMMARY_OK++))
else
    ((SUMMARY_ERROR++))
fi

if [ "$MARKER_PDF_FOUND" = true ]; then
    ((SUMMARY_OK++))
else
    ((SUMMARY_ERROR++))
fi

if [ -d "$HF_CACHE_DIR" ] && [ "$(get_file_count "$HF_CACHE_DIR")" -gt 0 ]; then
    ((SUMMARY_OK++))
else
    ((SUMMARY_WARN++))
fi

if [ -d "$SURYA_CACHE_DIR" ] && [ "$(get_file_count "$SURYA_CACHE_DIR")" -gt 0 ]; then
    ((SUMMARY_OK++))
else
    ((SUMMARY_WARN++))
fi

echo "检查结果统计:" | tee -a "$OUTPUT_FILE"
echo "  ✓ 正常: $SUMMARY_OK" | tee -a "$OUTPUT_FILE"
echo "  ⚠ 警告: $SUMMARY_WARN" | tee -a "$OUTPUT_FILE"
echo "  ✗ 错误: $SUMMARY_ERROR" | tee -a "$OUTPUT_FILE"
echo "" | tee -a "$OUTPUT_FILE"

# 关键问题提示
if [ "$MARKER_PDF_FOUND" = false ]; then
    report "ERROR" "关键问题: marker-pdf 未安装，无法执行 PDF 转换"
    echo "  解决方案: 按照上述安装方法安装 marker-pdf" | tee -a "$OUTPUT_FILE"
fi

if [ ! -d "$SURYA_CACHE_DIR" ] || [ "$(get_file_count "$SURYA_CACHE_DIR")" -eq 0 ]; then
    report "WARN" "关键问题: Surya OCR 模型未下载，首次运行需要下载（约 2-3GB）"
    echo "  解决方案: 首次运行 marker 时会自动下载，请确保网络连接稳定" | tee -a "$OUTPUT_FILE"
fi

if [ ! -d "$HF_CACHE_DIR" ] || [ "$(get_file_count "$HF_CACHE_DIR")" -eq 0 ]; then
    report "WARN" "关键问题: Hugging Face 模型缓存为空，可能需要下载模型"
    echo "  解决方案: 首次运行 marker 时会自动下载，请确保网络连接稳定" | tee -a "$OUTPUT_FILE"
fi

echo "" | tee -a "$OUTPUT_FILE"
report "SECTION" "检查完成"

echo "报告已保存到: $OUTPUT_FILE" | tee -a "$OUTPUT_FILE"
echo "" | tee -a "$OUTPUT_FILE"
echo "提示: 可以将此报告与正常工作的机器上的报告进行比对，查找差异" | tee -a "$OUTPUT_FILE"
echo "" | tee -a "$OUTPUT_FILE"

