FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN mkdir -p data
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=builder /app/target/release/general_notifier .
COPY --from=builder /app/.env .
COPY --from=builder /app/data ./data
COPY --from=builder /app/migrations ./migrations
ENTRYPOINT ["/app/general_notifier"]
