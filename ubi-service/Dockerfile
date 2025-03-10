FROM rust:1.81 AS builder
WORKDIR /usr/src/xbank
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin ubi-service

FROM debian:buster-slim
COPY --from=builder /usr/src/xbank/target/release/ubi-service /usr/local/bin/ubi-service
EXPOSE 8084
CMD ["ubi-service"]
