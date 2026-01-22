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
    # tank_battle 是一个目录，需要将里面的内容复制到正确位置
    install -Dm755 "tank_battle/tank_battle" "$pkgdir/usr/bin/tank_battle"
    install -dm755 "$pkgdir/usr/share/$pkgname"
    cp -r tank_battle/assets tank_battle/levels "$pkgdir/usr/share/$pkgname/"
    install -Dm644 tank_battle/LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"

    # Icon
    install -Dm644 tank_battle/assets/enemy_tank/enemy_tank1.png "$pkgdir/usr/share/pixmaps/$pkgname.png"
}