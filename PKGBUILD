# Maintainer: kiberdans <kiberdans@yandex.ru>
pkgname=raskladka
pkgver=0.1.0
pkgrel=1
pkgdesc="Keyboard layout switcher (QWERTY ↔ ЙЦУКЕН) via double-press of a configurable key"
arch=('x86_64')
url="https://github.com/kiberdans/raskladka"
license=('MIT')
depends=('xdotool' 'xclip')
optdepends=('wl-clipboard: Wayland support'
            'wtype: Wayland support')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/kiberdans/raskladka/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')
provides=("$pkgname")
conflicts=("$pkgname")

build() {
    cd "$srcdir/$pkgname-$pkgver"
    cargo build --release --locked
}

package() {
    cd "$srcdir/$pkgname-$pkgver"
    install -Dm755 target/release/raskladka "$pkgdir/usr/bin/raskladka"
    install -Dm644 raskladka.desktop "$pkgdir/usr/share/applications/raskladka.desktop"
    install -Dm644 raskladka.service "$pkgdir/usr/lib/systemd/user/raskladka.service"
    install -Dm644 on.svg "$pkgdir/usr/share/icons/hicolor/scalable/apps/raskladka-on.svg"
    install -Dm644 off.svg "$pkgdir/usr/share/icons/hicolor/scalable/apps/raskladka-off.svg"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
