FROM rust:1.70 as builder
WORKDIR ./src/
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get -y install ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/cloudflare-dynamic-dns /usr/local/bin/cloudflare-dynamic-dns
CMD ["cloudflare-dynamic-dns"]