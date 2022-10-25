FROM rust

WORKDIR /usr/src/spectacles
COPY . .

RUN cargo install --path ./gateway
RUN cargo install --path ./brokers/json

COPY gateway/gateway.toml .

ENTRYPOINT spectacles-gateway | spectacles-json
