# ---- build stage ----
FROM rust:1.82 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# ---- runtime stage ----
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/splatcom /app/
EXPOSE 8080
CMD ["/app/splatcom"]