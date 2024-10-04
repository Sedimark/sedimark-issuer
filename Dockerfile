# ---------------------------------------------------
# 1 - Build Stage
# ---------------------------------------------------

FROM rust:1.81-alpine AS build
WORKDIR /usr/src/app
COPY . .
RUN  apk add --no-cache make musl-dev clang llvm gcc libc-dev clang-dev binutils g++ linux-headers libstdc++ libgcc libressl-dev
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=./target \
    cargo install --path ./server

# ---------------------------------------------------
# 2 - Deploy Stage
# ---------------------------------------------------

FROM alpine:latest
RUN  apk add --no-cache musl-dev gcc libstdc++ libgcc libressl-dev
COPY --from=build /usr/local/cargo/bin/issuer /usr/local/bin/issuer
EXPOSE 3213
CMD [ "issuer" , "--rpc-provider", "https://json-rpc.evm.testnet.shimmer.network", "--chain-id" , "1073" ] 