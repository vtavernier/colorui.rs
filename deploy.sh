#!/bin/bash
set -euxo pipefail

export PKG_CONFIG_DIR=
export PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig
export PKG_CONFIG_ALLOW_CROSS=1

# Build target
cargo build --release --target aarch64-unknown-linux-gnu

# Build dist
(cd web && yarnpkg run build)

# Send release archive
tar -cz colorui.service web/dist target/aarch64-unknown-linux-gnu/release/colorui run.sh | ssh -t odroidc2 'tar -xzv --one-top-level=colorui.rs ; sudo install colorui.rs/colorui.service /etc/systemd/system/colorui.service ; sudo systemctl daemon-reload ; sudo systemctl restart colorui.service'
