pkgname=batt
pkgver=1.1.0
pkgrel=1
pkgdesc='A simple battery monitor.'
arch=('i686' 'x86_64')
license=('ISC')
makedepends=('rust' 'cargo')
url="https://github.com/wvauclain/batt"
source=("https://github.com/wvauclain/batt/archive/v$pkgver.tar.gz")
sha256sums=(SKIP)

build() {
  cd $srcdir/batt-$pkgver
  cargo build --release
  strip target/release/batt
}

package() {
  cd $srcdir/batt-$pkgver
  install -Dm755 target/release/batt "$pkgdir/usr/local/bin/batt"
}
