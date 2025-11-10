#!/bin/bash
# Marker 工具命令行测试脚本
# 用于验证 marker_single 命令是否能够正常工作

set -e

echo "=========================================="
echo "Marker 工具命令行测试"
echo "=========================================="
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 配置
APP_PATH="/Users/niuzhidao/Library/Developer/Xcode/DerivedData/Runner-dyciyghacflcaiailhbvokbydexe/Build/Products/Debug/AppFlowy.app"
MARKER_TOOL="${APP_PATH}/Contents/Resources/marker/marker"

# 测试 PDF 文件路径（如果提供的话）
TEST_PDF="${1:-/Volumes/Macintosh HD/Users/niuzhidao/Downloads/pdf 导入文件 v2.pdf}"

# 输出目录
OUTPUT_DIR="/tmp/test_marker_output_$(date +%s)"
mkdir -p "$OUTPUT_DIR"

echo "配置信息:"
echo "  Marker 工具路径: $MARKER_TOOL"
echo "  测试 PDF 文件: $TEST_PDF"
echo "  输出目录: $OUTPUT_DIR"
echo ""

# 检查 Marker 工具是否存在
if [ ! -f "$MARKER_TOOL" ]; then
    echo -e "${RED}✗ Marker 工具不存在: $MARKER_TOOL${NC}"
    exit 1
fi

# 检查 Marker 工具是否可执行
if [ ! -x "$MARKER_TOOL" ]; then
    echo -e "${YELLOW}⚠ Marker 工具不可执行，尝试添加执行权限...${NC}"
    chmod +x "$MARKER_TOOL"
fi

# 检查测试 PDF 文件是否存在
if [ ! -f "$TEST_PDF" ]; then
    echo -e "${YELLOW}⚠ 测试 PDF 文件不存在: $TEST_PDF${NC}"
    echo "  请提供 PDF 文件路径作为第一个参数"
    echo "  用法: $0 <pdf_file_path>"
    exit 1
fi

echo -e "${GREEN}✓ 所有检查通过${NC}"
echo ""

# 测试 1: 检查 marker 工具是否能够显示帮助信息
echo "=========================================="
echo "测试 1: 检查 marker 工具帮助信息"
echo "=========================================="
if timeout 5 "$MARKER_TOOL" --help > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Marker 工具可以显示帮助信息${NC}"
else
    echo -e "${RED}✗ Marker 工具无法显示帮助信息（可能卡住或失败）${NC}"
    exit 1
fi
echo ""

# 测试 2: 执行实际的 PDF 转换命令（模拟 Rust 代码中的调用）
echo "=========================================="
echo "测试 2: 执行 PDF 转换命令"
echo "=========================================="
echo "命令:"
echo "  $MARKER_TOOL \\"
echo "    \"$TEST_PDF\" \\"
echo "    --output_dir \"$OUTPUT_DIR\" \\"
echo "    --output_format markdown"
echo ""
echo "注意: marker_single 默认不使用 LLM（除非配置了 LLM API key）"
echo "注意: 首次运行需要下载模型文件，可能需要较长时间（5-10 分钟）"
echo ""

# 检查模型是否已下载
MODEL_CACHE_DIR="$HOME/Library/Caches/datalab/models"
if [ -d "$MODEL_CACHE_DIR" ]; then
    echo "模型缓存目录存在: $MODEL_CACHE_DIR"
    MODEL_COUNT=$(find "$MODEL_CACHE_DIR" -type f 2>/dev/null | wc -l | tr -d ' ')
    echo "已缓存模型文件数: $MODEL_COUNT"
    if [ "$MODEL_COUNT" -gt 0 ]; then
        echo -e "${GREEN}✓ 模型已下载，转换应该较快${NC}"
    else
        echo -e "${YELLOW}⚠ 模型未下载，首次运行需要下载（可能需要 5-10 分钟）${NC}"
    fi
else
    echo -e "${YELLOW}⚠ 模型缓存目录不存在，首次运行需要下载（可能需要 5-10 分钟）${NC}"
fi
echo ""

echo "开始执行转换（超时时间: 1800 秒 = 30 分钟，首次运行需要下载多个大型模型）..."
echo "注意: 首次运行需要下载约 2-3GB 的模型文件，请确保网络连接稳定"
echo ""

# 执行命令并捕获输出
# 首次运行需要下载多个大型模型（layout、text_recognition 等），每个约 1-2GB
# 总下载时间可能需要 15-30 分钟，取决于网络速度
START_TIME=$(date +%s)
if timeout 1800 "$MARKER_TOOL" \
    "$TEST_PDF" \
    --output_dir "$OUTPUT_DIR" \
    --output_format markdown \
    > "$OUTPUT_DIR/stdout.log" 2> "$OUTPUT_DIR/stderr.log"; then
    
    EXIT_CODE=$?
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    
    echo -e "${GREEN}✓ 命令执行完成（退出码: $EXIT_CODE，耗时: ${DURATION} 秒）${NC}"
    echo ""
    
    # 显示输出
    echo "标准输出 (stdout):"
    echo "----------------------------------------"
    if [ -s "$OUTPUT_DIR/stdout.log" ]; then
        cat "$OUTPUT_DIR/stdout.log"
    else
        echo "(空)"
    fi
    echo ""
    
    echo "标准错误 (stderr):"
    echo "----------------------------------------"
    if [ -s "$OUTPUT_DIR/stderr.log" ]; then
        cat "$OUTPUT_DIR/stderr.log"
    else
        echo "(空)"
    fi
    echo ""
    
    # 检查输出目录中的文件
    echo "输出目录内容:"
    echo "----------------------------------------"
    ls -lah "$OUTPUT_DIR" | head -20
    echo ""
    
    # 查找生成的 Markdown 文件
    MARKDOWN_FILES=$(find "$OUTPUT_DIR" -name "*.md" -o -name "*.markdown" 2>/dev/null)
    if [ -n "$MARKDOWN_FILES" ]; then
        echo -e "${GREEN}✓ 找到生成的 Markdown 文件:${NC}"
        echo "$MARKDOWN_FILES" | while read -r file; do
            echo "  - $file ($(wc -c < "$file" | tr -d ' ') 字节)"
        done
        echo ""
        
        # 显示第一个 Markdown 文件的前几行
        FIRST_MD=$(echo "$MARKDOWN_FILES" | head -1)
        if [ -n "$FIRST_MD" ]; then
            echo "Markdown 文件预览 (前 30 行):"
            echo "----------------------------------------"
            head -30 "$FIRST_MD"
            echo ""
        fi
    else
        echo -e "${YELLOW}⚠ 未找到生成的 Markdown 文件${NC}"
        echo ""
    fi
    
    if [ $EXIT_CODE -eq 0 ]; then
        echo -e "${GREEN}=========================================="
        echo "✓ 测试成功！"
        echo "==========================================${NC}"
        exit 0
    else
        echo -e "${RED}=========================================="
        echo "✗ 测试失败（退出码: $EXIT_CODE）"
        echo "==========================================${NC}"
        exit 1
    fi
else
    EXIT_CODE=$?
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    
    echo -e "${RED}✗ 命令执行失败或超时（退出码: $EXIT_CODE，耗时: ${DURATION} 秒）${NC}"
    echo ""
    
    # 显示错误输出
    echo "标准输出 (stdout):"
    echo "----------------------------------------"
    if [ -s "$OUTPUT_DIR/stdout.log" ]; then
        cat "$OUTPUT_DIR/stdout.log"
    else
        echo "(空)"
    fi
    echo ""
    
    echo "标准错误 (stderr):"
    echo "----------------------------------------"
    if [ -s "$OUTPUT_DIR/stderr.log" ]; then
        cat "$OUTPUT_DIR/stderr.log"
    else
        echo "(空)"
    fi
    echo ""
    
    echo -e "${RED}=========================================="
    echo "✗ 测试失败"
    echo "==========================================${NC}"
    exit 1
fi

