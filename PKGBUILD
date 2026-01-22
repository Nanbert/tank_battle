# Maintainer: Nanbert <nanbert@example.com>
pkgname=tank-battle
pkgver=0.2.0
pkgrel=1
pkgdesc="Classic tank battle game (Battle City 1990 clone)"
arch=('x86_64')
url="https://github.com/Nanbert/tank_battle"
license=('MIT')
depends=('gcc-libs' 'glibc')
makedepends=('cargo')
source=()
sha256sums=()

prepare() {
    export RUSTUP_TOOLCHAIN=nightly
    cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
    export RUSTUP_TOOLCHAIN=nightly
    export CARGO_TARGET_DIR=target
    cargo build --release --frozen
}

check() {
    export RUSTUP_TOOLCHAIN=nightly
    cargo test --frozen
}

package() {
    install -Dm755 "tank_battle" "$pkgdir/usr/bin/tank_battle"
    install -dm755 "$pkgdir/usr/share/$pkgname"
    cp -r assets levels "$pkgdir/usr/share/$pkgname/"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"

    # Icon
    install -Dm644 assets/enemy_tank/enemy_tank1.png "$pkgdir/usr/share/pixmaps/$pkgname.png"
}