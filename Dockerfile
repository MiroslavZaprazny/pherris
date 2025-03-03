FROM rust:latest AS base

RUN apt-get update && apt-get install -y php php-cli

WORKDIR /app
COPY . .
RUN cargo build

FROM base AS test
CMD ["cargo", "test"]

FROM base AS run
CMD ["cargo", "run"]
