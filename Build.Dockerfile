FROM alpine:3.19.1

# Install required packages
RUN apk add --no-cache \
    build-base \
    curl \
    git \
    openssl-dev \
    perl \
    rustup

# Install Rust toolchain
RUN rustup-init -y --default-toolchain stable && \
    rm -rf /root/.cargo/registry && \
    rm -rf /root/.cargo/git

# Set environment variables
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY . .

CMD ["cargo", "build", "--release"]