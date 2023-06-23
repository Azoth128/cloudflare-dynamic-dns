FROM rust:1.70

WORKDIR ./src/
COPY . .

RUN cargo install --path .

CMD ["cloudflare-dynamic-dns"]