FROM rust:latest AS base

WORKDIR /app
COPY . .
RUN cargo build

FROM base AS test
CMD ["cargo", "test"]

FROM base AS run
CMD ["cargo", "run"]
