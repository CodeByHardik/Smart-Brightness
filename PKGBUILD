# Maintainer: Hardik Dubey <CodeByHardik>
pkgname=smart-brightness
pkgver=0.3.0
pkgrel=1
pkgdesc="Automatic screen brightness adjustment based on ambient light"
arch=('x86_64')
url="https://github.com/CodeByHardik/Smart-Brightness"
license=('MIT')
depends=('glibc' 'gcc-libs')
makedepends=('cargo')
provides=('smart-brightness')
conflicts=('smart-brightness')
source=()
options=('!strip')

prepare() {
    # Check if we are running in the project root
    if [ ! -f "${startdir}/Cargo.toml" ]; then
        echo "Error: PKGBUILD must be run from the project root."
        exit 1
    fi
}

build() {
    cd "${startdir}"
    cargo build --release --locked
}

package() {
    cd "${startdir}"
    install -Dm755 target/release/smart-brightness "$pkgdir/usr/bin/smart-brightness"
    install -Dm644 smart-brightnessd.service "$pkgdir/usr/lib/systemd/system/smart-brightnessd.service"
}
