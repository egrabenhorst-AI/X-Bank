FROM rust:1.81 AS builder
WORKDIR /usr/src/xbank
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin account-service

FROM debian:buster-slim
COPY --from=builder /usr/src/xbank/target/release/account-service /usr/local/bin/account-service
EXPOSE 8082
CMD ["account-service"]
