FROM debian:stretch-slim
MAINTAINER Cedric Dumay

################################################################################
## Image / System variables
ENV SERIAL=1 \
    DEBIAN_FRONTEND=noninteractive \
    PATH=/usr/local/cargo/bin:$PATH

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
    gcc \
    git \
    libc-dev \
    libc6-dev \
    liblzma-dev \
    libsnappy-dev \
    libtool \
    make \
    patch \
    pkg-config \
    zlib1g-dev

RUN set -eux; \
    apt-get update -qq; \
    apt-get install -qq ${BUILD_DEPS} --no-install-recommends

################################################################################
## LibreSSL install
ENV LIBRESSL_VERSION 2.7.5
ENV LIBRESSL_DOWNLOAD_URL https://ftp.openbsd.org/pub/OpenBSD/LibreSSL/libressl-${LIBRESSL_VERSION}.tar.gz

RUN set -x && \
    mkdir -p /tmp/src && \
    cd /tmp/src && \
    curl -sSL $LIBRESSL_DOWNLOAD_URL -o libressl.tar.gz && \
    tar xzf libressl.tar.gz && \
    rm -f libressl.tar.gz && \
    cd libressl-${LIBRESSL_VERSION} && \
    ./configure --disable-shared --with-pic --disable-dependency-tracking --prefix=/opt/libressl && \
    make check && make install && \
    rm -fr /opt/libressl/share/man && \
    echo /opt/libressl/lib > /etc/ld.so.conf.d/libressl.conf && ldconfig

ENV OPENSSL_LIB_DIR=/opt/libressl/lib
ENV OPENSSL_INCLUDE_DIR=/opt/libressl/include

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
    git clone https://github.com/cdumay/flowgger && \
    cd flowgger && \
    cargo build --release --features='coroutines kafka' && \
    mkdir -p /opt/flowgger/etc /opt/flowgger/bin && \
    strip target/release/flowgger && \
    mv target/release/flowgger /opt/flowgger/bin/

################################################################################
## cleaning
RUN apt-get clean; \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* /opt/libressl/share/man

ENV LD_LIBRARY_PATH=${LD_LIBRARY_PATH}:/usr/local/lib \
    RUST_LOG=trace

EXPOSE 6514
ENTRYPOINT ["/opt/flowgger/bin/flowgger"]
