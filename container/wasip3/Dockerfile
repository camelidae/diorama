FROM golang:1.27.2-alpine AS containerd
RUN apk add --no-cache make git curl patch
WORKDIR /src
COPY container/wasip3/build-containerd.sh container/wasip3/containerd.patch /build/
RUN sh /build/build-containerd.sh

FROM rust:1.97.1-alpine AS wasip3
RUN apk add --no-cache build-base clang-dev llvm-dev cmake pkgconf \
    libseccomp-dev libseccomp-static protobuf-dev git perl linux-headers curl
WORKDIR /src
COPY container/wasip3/build.sh container/wasip3/runwasi.patch /build/
RUN sh /build/build.sh

FROM scratch
COPY --from=containerd /out/ /bin/
COPY --from=wasip3 /out/ /bin/
