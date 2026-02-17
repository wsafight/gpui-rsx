#!/bin/bash
# =============================================================================
# Code Coverage Report Generator
# 代码覆盖率报告生成器
# =============================================================================
#
# English:
# Generates code coverage report using cargo-llvm-cov.
# Outputs both terminal summary and HTML report.
#
# Usage:
#   ./coverage.sh          # Generate HTML report
#   ./coverage.sh --text   # Show terminal output only
#
# 中文：
# 使用 cargo-llvm-cov 生成代码覆盖率报告。
# 输出终端摘要和 HTML 报告。
#
# 用法：
#   ./coverage.sh          # 生成 HTML 报告
#   ./coverage.sh --text   # 仅显示终端输出
# =============================================================================

set -e

# Check if cargo-llvm-cov is installed
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "❌ cargo-llvm-cov not found. Installing..."
    cargo install cargo-llvm-cov
fi

# Parse arguments
if [ "$1" == "--text" ]; then
    echo "📊 Generating coverage report (text only)..."
    cargo llvm-cov --all-features --workspace
else
    echo "📊 Generating coverage report..."
    cargo llvm-cov --all-features --workspace --html
    echo ""
    echo "✅ Coverage report generated!"
    echo "📄 Open: target/llvm-cov/html/index.html"

    # Try to open in browser on macOS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        open target/llvm-cov/html/index.html
    fi
fi
