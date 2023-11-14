
FROM rust:buster as builder
WORKDIR /usr/src/gateway

COPY . .

RUN cargo install --path .

FROM debian:buster-slim as runtime

RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/api-gateway /usr/local/bin/api-gateway
COPY --from=builder /usr/src/gateway/config.yaml config.yaml

EXPOSE 8080

CMD ["api-gateway"]