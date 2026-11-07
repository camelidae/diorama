#!/bin/sh
# Pinned runwasi with the WASI 0.3-only Wasmtime adapter.
set -eu
revision=f1401719a3e981956ced9cd685abb2a49190bd1c
curl --fail --location --retry 3 "https://github.com/containerd/runwasi/archive/${revision}.tar.gz" -o /tmp/runwasi.tar.gz
echo '1169cd27635c831c20727046b608d7f9c1590b7272c2a4c7ef4f9d73e1abe308  /tmp/runwasi.tar.gz' | sha256sum -c -
tar -xzf /tmp/runwasi.tar.gz --strip-components=1 -C /src
cd /src
patch -p1 < /build/runwasi.patch
# Keep the development-image build small; this stage is cached across app edits.
export RUSTUP_TOOLCHAIN=1.97.1 CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_DEV_INCREMENTAL=false
cargo build --locked -p containerd-shim-wasmtime --bin containerd-shim-wasmtime-v1
mkdir -p /out
cp target/debug/containerd-shim-wasmtime-v1 /out/
strip /out/containerd-shim-wasmtime-v1

# Fail packaging if a toolchain change introduces shared-library dependencies.
if readelf -l /out/containerd-shim-wasmtime-v1 | grep -q INTERP; then
    echo "WASI shim must be statically linked" >&2
    exit 1
fi
# Keep compiler caches out of the image layer (the output above is self-contained).
rm -rf /src/target /usr/local/cargo/registry /usr/local/cargo/git /tmp/runwasi.tar.gz
