# colorui.rs

Status: prototype, not made for production use.

## Usage

Install web dependencies:

```bash
cd web && yarnpkg install
```

### Development

```bash
# Start websocket server (forwards requests to serial port)
cargo run

# Start development web UI
cd web && yarnpkg run serve
```

### Production

```bash
# Build release version
cargo build --release

# Build assets
cd web && yarnpkg run build

# Run serve (will serve assets from web/dist)
./target/release/colorui
```

### Cross-compiling (aarch64)

```bash
# Cross-compile dependencies
rustup target add --toolchain stable aarch64-unknown-linux-gnu
sudo apt install aarch64-linux-gnu-gcc

# Add libudev build dependencies
sudo dpkg --add-architecture arm64
sudo apt install libudev-dev:arm64

# Configure pkg-config
export PKG_CONFIG_DIR=
export PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig
export PKG_CONFIG_ALLOW_CROSS=1

# Build target
cargo build --release --target aarch64-unknown-linux-gnu

# Build assets
cd web && yarnpkg run build

# Move binaries and assets to target
tar cvf archive.tar web/dist target/aarch64-unknown-linux-gnu/release/colorui run.sh

# Run (on target)
./run.sh
```

## Author

Vincent Tavernier <vince.tavernier@gmail.com>
