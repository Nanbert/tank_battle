# Maintainer: Nanbert <nanbert@example.com>
pkgname=tank-battle
pkgver=0.2.0
pkgrel=1
pkgdesc="Classic tank battle game (Battle City 1990 clone)"
arch=('x86_64')
url="https://github.com/Nanbert/tank_battle"
license=('MIT')
depends=('gcc-libs' 'glibc')
makedepends=()
source=()
sha256sums=()

# 预编译的二进制包，由 CI 提供
build() {
    # 跳过编译，直接使用已编译的二进制文件
    echo "Using pre-compiled binary from CI"
}

check() {
    # 跳过测试
    echo "Skipping tests for pre-compiled binary"
}

package() {
    # 调试信息：检查当前目录和文件
    echo "=== DEBUG INFO ==="
    echo "Current directory: $(pwd)"
    echo "Files in current directory:"
    ls -la
    echo "Files in ../tank_battle directory:"
    ls -la ../tank_battle/ 2>/dev/null || echo "../tank_battle directory not found!"
    echo "=================="

    # makepkg 在 src/ 目录下运行，需要使用 ../tank_battle/ 访问根目录的文件
    install -Dm755 "../tank_battle/tank_battle" "$pkgdir/usr/bin/tank_battle"
    install -dm755 "$pkgdir/usr/share/$pkgname"
    cp -r ../tank_battle/assets ../tank_battle/levels "$pkgdir/usr/share/$pkgname/"
    install -Dm644 ../tank_battle/LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"

    # Icon
    install -Dm644 "../tank_battle/assets/enemy_tank/enemy_tank1.png" "$pkgdir/usr/share/pixmaps/$pkgname.png"
}