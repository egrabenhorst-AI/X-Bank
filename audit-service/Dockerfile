FROM rust:1.81 AS builder
WORKDIR /usr/src/xbank
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin audit-service

FROM debian:buster-slim
COPY --from=builder /usr/src/xbank/target/release/audit-service /usr/local/bin/audit-service
EXPOSE 8085
CMD ["audit-service"]
