#!/bin/bash
# Steel Command Web 构建脚本

set -e

echo "🔨 开始构建 Steel Command Web 版本..."

# 检查 wasm32 目标是否已安装
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "📦 安装 wasm32-unknown-unknown 目标..."
    rustup target add wasm32-unknown-unknown
fi

# 检查 wasm-pack 是否已安装
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack 未安装，请先安装："
    echo "   cargo install wasm-pack"
    exit 1
fi

# 创建输出目录
mkdir -p dist

# 构建优化版本的 wasm
echo "🔧 构建 WebAssembly 模块..."
wasm-pack build --release --target web --out-dir dist

# 复制 HTML 和资源文件
echo "📋 复制文件..."
cp index.html dist/
cp -r assets dist/

# 复制关卡文件
mkdir -p dist/levels
cp -r levels/* dist/levels/

# 优化 wasm 体积
if command -v wasm-opt &> /dev/null; then
    echo "⚡ 优化 wasm 文件体积..."
    wasm-opt -Oz dist/tank_battle_bg.wasm -o dist/tank_battle_bg.wasm.optimized
    mv dist/tank_battle_bg.wasm.optimized dist/tank_battle_bg.wasm
fi

echo "✅ 构建完成！"
echo "📂 输出目录: dist/"
echo "🚀 要运行 Web 版本，请在 dist 目录下启动一个 HTTP 服务器："
echo "   cd dist && python -m http.server 8000"
echo "   或"
echo "   cd dist && npx http-server -p 8000"
echo ""
echo "🌐 然后在浏览器中打开: http://localhost:8000"