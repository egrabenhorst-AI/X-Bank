FROM rust:1.81 AS builder
WORKDIR /usr/src/xbank
COPY . .
RUN cargo build --release --bin auth-service

FROM debian:buster-slim
COPY --from=builder /usr/src/xbank/target/release/auth-service /usr/local/bin/auth-service
EXPOSE 8081
CMD ["auth-service"]
