# ---------------------------------------------------
# 1 - Build Stage
# ---------------------------------------------------

FROM rustlang/rust:nightly-alpine AS build
WORKDIR /usr/src/app
COPY . .
RUN  apk add --no-cache make musl-dev clang llvm gcc libc-dev clang-dev binutils g++ linux-headers libstdc++ libgcc libressl-dev
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cd abigen  \ 
    && cargo run -- --contract Identity --abi-source "../smart-contracts/Identity.json" \
    && cd ..
RUN cargo install --path ./server

# ---------------------------------------------------
# 2 - Deploy Stage
# ---------------------------------------------------

FROM alpine:latest
RUN  apk add --no-cache make musl-dev clang llvm gcc libc-dev clang-dev binutils g++ linux-headers libstdc++ libgcc libressl-dev
COPY --from=build /usr/local/cargo/bin/issuer /usr/local/bin/issuer
COPY --from=build /usr/src/app/server/.env /.env
EXPOSE 3213
CMD [ "issuer" , "--rpc-provider", "https://json-rpc.evm.testnet.shimmer.network", "--chain-id" , "1073" ] 