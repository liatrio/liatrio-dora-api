FROM alpine:3.19.1

# Install required packages
RUN apk add --no-cache \
    build-base \
    curl \
    git \
    openssl-dev \
    perl \
    rustup \
    pkgconfig \
    musl-dev

# Install Rust toolchain
RUN rustup-init -y --default-toolchain stable && \
    rustup target add x86_64-unknown-linux-musl && \
    rm -rf /root/.cargo/registry && \
    rm -rf /root/.cargo/git

# Set environment variables
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY . .

CMD ["cargo", "build", "--release", "--target", "x86_64-unknown-linux-musl"]