FROM rust:1.81 AS builder
WORKDIR /usr/src/xbank
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin transaction-service

FROM debian:buster-slim
COPY --from=builder /usr/src/xbank/target/release/transaction-service /usr/local/bin/transaction-service
EXPOSE 8083
CMD ["transaction-service"]
