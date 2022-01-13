ARG BASE_IMAGE=rust:1.58

FROM $BASE_IMAGE as builder
WORKDIR app
ENV SQLX_OFFLINE='true'
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
COPY ./sqlx-data.json ./sqlx-data.json
RUN mkdir -p data
RUN cargo build --release

FROM gcr.io/distroless/cc-debian11
COPY --from=builder /app/target/release/general_notifier /
COPY --from=builder /app/data /
COPY ./migrations ./migrations
COPY ./.env ./.env
CMD ["./general_notifier"]
