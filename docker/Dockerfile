FROM debian:stretch-slim
MAINTAINER Cedric Dumay

################################################################################
## Image / System variables
ENV SERIAL=1 \
    DEBIAN_FRONTEND=noninteractive \
    PATH=/usr/local/cargo/bin:$PATH \
    LD_LIBRARY_PATH=${LD_LIBRARY_PATH}:/usr/local/lib

################################################################################
## OS install
ENV BUILD_DEPS \
    autoconf \
    automake \
    ca-certificates \
    curl \
    file \
    g++ \
    gcc \
    git \
    libc-dev \
    libc6-dev \
    libtool \
    make \
    cmake \
    patch \
    pkg-config

RUN set -eux; \
    apt-get update -qq; \
    apt-get install -qq ${BUILD_DEPS} --no-install-recommends

################################################################################
## zlib install
ENV ZLIB_VERSION 1.2.11
RUN set -x && \
    cd /tmp && \
    curl -sSL "http://zlib.net/zlib-${ZLIB_VERSION}.tar.gz" | tar xz && \
    cd zlib-${ZLIB_VERSION} && \
    ./configure && \
    CFLAGS="-O3 -fPIC" make && \
    make install

################################################################################
## snappy install
ENV SNAPPY_VERSION 1.1.7
RUN set -x && \
    cd /tmp && \
    curl -sSL "https://github.com/google/snappy/archive/${SNAPPY_VERSION}.tar.gz" | tar xz && \
    cd snappy-${SNAPPY_VERSION} && \
    mkdir build && \
    cd build && \
    cmake ../ && \
    CFLAGS="-O3 -fPIC" make

################################################################################
## liblzma install
ENV XZ_VERSION 5.2.4
RUN set -x && \
    cd /tmp && \
    curl -sSL "https://netcologne.dl.sourceforge.net/project/lzmautils/xz-${XZ_VERSION}.tar.gz" | tar xz && \
    cd xz-${XZ_VERSION} && \
    ./configure && \
    cd lib && \
    CFLAGS="-O3 -fPIC" make && \
    make install

################################################################################
## LZ4 static & fPIC
ENV LZ4_VERSION=1.8.3
RUN set -x && \
    cd /tmp && \
    curl -sSL "https://github.com/lz4/lz4/archive/v${LZ4_VERSION}.tar.gz" | tar xz && \
    cd lz4-${LZ4_VERSION}/lib && \
    CFLAGS="-O3 -fPIC" make && \
    make install

################################################################################
## ZSTD static & fPIC
ENV ZSTD_VERSION=1.3.8
RUN set -x && \
    cd /tmp && \
    curl -sSL "https://github.com/facebook/zstd/archive/v${ZSTD_VERSION}.tar.gz" | tar xz && \
    cd zstd-${ZSTD_VERSION}/lib && \
    CFLAGS="-O3 -fPIC" make && \
    make install

################################################################################
## OpenSSL install
ENV OPENSSL_VERSION 1.1.1a
RUN set -x && \
    mkdir -p /tmp/src && \
    cd /tmp/src && \
    curl -sSL "https://www.openssl.org/source/openssl-${OPENSSL_VERSION}.tar.gz"| tar xz && \
    cd openssl-${OPENSSL_VERSION} && \
    ./config && \
    CFLAGS="-O3 -fPIC" make && \
    make install_sw && \
    rm -fr /usr/local/share/doc/openssl

################################################################################
## RUST install
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo

RUN mkdir -p "${RUSTUP_HOME}" "${CARGO_HOME}" && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --no-modify-path --default-toolchain nightly && \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME && \
    rustup --version && \
    cargo --version && \
    rustc --version

################################################################################
## librdKafka
RUN set -x && \
    cd /tmp && \
    git clone https://github.com/edenhill/librdkafka

RUN cd /tmp/librdkafka && \
    ./configure --enable-static && \
    make libs && \
    make install

#################################################################################
### Flowgger build & install
RUN set -x && \
    cd /tmp && \
    git clone --single-branch --branch openssl-111 https://github.com/cdumay/flowgger && \
    cd flowgger && \
    cargo build --release --features='coroutines kafka' && \
    mkdir -p /opt/flowgger/etc /opt/flowgger/bin && \
    strip target/release/flowgger && \
    mv target/release/flowgger /opt/flowgger/bin/

################################################################################
## cleaning
RUN apt-get clean; \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* /opt/libressl/share/man

EXPOSE 6514
ENTRYPOINT ["/opt/flowgger/bin/flowgger"]
