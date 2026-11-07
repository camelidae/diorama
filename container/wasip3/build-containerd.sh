#!/bin/sh
# Upstream runtime-handler pulls (PR #14314), plus platform-preserving creation.
# Replace with a checksum-pinned release once the fix ships.
set -eu
revision=43aa4ea134f3e50791b8537c48a18303f0fcc775
curl --fail --location --retry 3 "https://github.com/containerd/containerd/archive/${revision}.tar.gz" -o /tmp/containerd.tar.gz
echo '692108f62192c827f0d7c56da20cf50fcbd478f4954d397544309bc1d1513805  /tmp/containerd.tar.gz' | sha256sum -c -
tar -xzf /tmp/containerd.tar.gz --strip-components=1 -C /src
cd /src
patch -p1 < /build/containerd.patch
CGO_ENABLED=0 make VERSION=v2.5.0-dev REVISION="${revision}.p3" DEBUG= STATIC=1 GO_BUILD_FLAGS=-p=4 bin/containerd bin/containerd-shim-runc-v2 bin/ctr
mkdir -p /out
cp bin/containerd bin/containerd-shim-runc-v2 bin/ctr /out/
