# ---- build stage ----
FROM rust:1.95 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build

# ---- runtime stage ----
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/debug/splatcom /app/
EXPOSE 8080
ENTRYPOINT ["/app/splatcom"]
CMD ["/app/splatcom"]