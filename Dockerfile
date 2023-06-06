FROM rust:latest as builder

WORKDIR /usr/src/app

COPY . /usr/src/app

RUN cargo build --release

FROM debian:buster-slim

COPY --from=builder /usr/src/app/target/release/discord-bot /usr/src/app/target/release/discord-bot

RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

CMD [ "/usr/src/app/target/release/discord-bot" ]
