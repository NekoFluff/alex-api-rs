FROM rust:1.71 AS builder
COPY . .
RUN cargo build

FROM debian:bullseye
COPY --from=builder ./target/debug/alex-api-rs ./target/debug/alex-api-rs
CMD ["./target/debug/alex-api-rs"]