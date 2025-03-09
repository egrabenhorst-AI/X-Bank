FROM rust:1.81 AS builder
WORKDIR /usr/src/app
COPY . .
ENV DATABASE_URL=postgres://user:password@db:5432/xbank
ENV SQLX_OFFLINE=true
RUN cargo clean
RUN cargo build --release

FROM debian:buster-slim AS runtime
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/payment-service /usr/local/bin/payment-service
EXPOSE 3002
CMD ["payment-service"]