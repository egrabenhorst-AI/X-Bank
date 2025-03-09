FROM rust:1.81 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo clean
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder /usr/src/app/target/release/frontend-service /usr/local/bin/
EXPOSE 3004
CMD ["frontend-service"]