FROM docker.io/rust:alpine as build

# Install required packages
RUN apk add --no-cache \
    build-base \
    curl \
    git \
    openssl-dev \
    perl \
    rustup \
    pkgconfig \
    musl-dev \
    clang

# Install Rust toolchain
RUN rustup-init -y --default-toolchain stable
RUN rm -rf /root/.cargo/registry
RUN rm -rf /root/.cargo/git

# Set environment variables
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY . .

RUN cargo build --release

FROM alpine:latest

EXPOSE 3000
WORKDIR /app
COPY --from=build /app/target/release/liatrio-dora-api /app

ENTRYPOINT ["./liatrio-dora-api"]
