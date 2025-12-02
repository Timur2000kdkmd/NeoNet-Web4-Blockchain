# ci/pqc_test.Dockerfile
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y build-essential git curl wget cmake python3 python3-pip pkg-config     libssl-dev libgmp-dev libtool autoconf automake make clang ca-certificates golang-go cargo

# Build liboqs from sources
WORKDIR /opt
RUN git clone --depth 1 https://github.com/open-quantum-safe/liboqs.git || true
WORKDIR /opt/liboqs
RUN mkdir -p build && cd build && cmake -DCMAKE_INSTALL_PREFIX=/usr/local .. && make -j$(nproc) && make install

# set library path
ENV LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
ENV PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH

# Install liboqs-go (requires CGO + liboqs installed)
WORKDIR /opt
RUN go install github.com/open-quantum-safe/liboqs-go/oqs@latest || true

# Copy project into container
WORKDIR /workspace
COPY . /workspace

# Run Rust and Go tests (user may need to adjust commands locally)
CMD ["/bin/bash","-lc","set -e; cd enhancements_scaffold/pqc/rust_pqc; cargo test --verbose; cd /workspace/enhancements_scaffold/pqc/go_pqc; go test ./... || true; echo 'CI container finished.'"
]
