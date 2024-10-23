# ---------------------------------------------------
# 1 - Build Stage
# ---------------------------------------------------

FROM rust:1.81-alpine AS issuer-build
WORKDIR /app
COPY ./server .
COPY ./smart-contracts /smart-contracts
RUN  apk add --no-cache make musl-dev clang llvm gcc libc-dev clang-dev binutils g++ linux-headers libstdc++ libgcc libressl-dev
# Since the cache is unmounted, I need to move the generated executable in another place
RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --release && \
    cp target/release/issuer /app/issuer

# ---------------------------------------------------
# 2 - Deploy Stage
# ---------------------------------------------------

FROM alpine:latest
COPY --from=issuer-build /app/issuer /usr/local/bin/issuer
EXPOSE 3213
CMD ["issuer"]