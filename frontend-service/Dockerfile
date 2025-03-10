FROM rust:1.81 AS builder
WORKDIR /usr/src/xbank
COPY . .
RUN cargo build --release --bin frontend-service

FROM debian:buster-slim
COPY --from=builder /usr/src/xbank/target/release/frontend-service /usr/local/bin/frontend-service
EXPOSE 8080
CMD ["frontend-service"]
