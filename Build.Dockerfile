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
RUN rustup-init -y --default-toolchain stable
RUN rm -rf /root/.cargo/registry
RUN rm -rf /root/.cargo/git

# Set environment variables
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY . .

CMD ["cargo", "build", "--release"]