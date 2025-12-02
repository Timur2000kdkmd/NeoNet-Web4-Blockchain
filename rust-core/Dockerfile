FROM rust:1.70-slim
WORKDIR /usr/src/neonet
COPY . .
RUN cargo build --release
CMD ["./target/release/neonet_core"]