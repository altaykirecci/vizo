# Maintainer: Altay Kireççi
pkgname=vizo
pkgver=0.1.3
pkgrel=1
pkgdesc="High performance image and camera RAW viewer written in Rust"
arch=('x86_64')
url="https://github.com/altaykirecci/vizo"
license=('MIT' 'Apache-2.0')
depends=('gcc-libs' 'glibc')
makedepends=('cargo')

build() {
    cargo build --release --locked
}

package() {
    install -Dm755 "$startdir/target/release/vizo" "$pkgdir/usr/bin/vizo"
    install -Dm644 "$startdir/vizo.desktop" "$pkgdir/usr/share/applications/vizo.desktop"
    install -Dm644 "$startdir/assets/vizo.svg" "$pkgdir/usr/share/icons/hicolor/scalable/apps/vizo.svg"
    install -Dm644 "$startdir/README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
}
