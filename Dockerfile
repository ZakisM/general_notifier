ARG BASE_IMAGE=rust:1.52.1
ARG BASE_CHEF_IMAGE=lukemathwalker/cargo-chef:latest

FROM $BASE_CHEF_IMAGE as planner
WORKDIR app
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM $BASE_CHEF_IMAGE as cacher
WORKDIR app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM $BASE_IMAGE as builder
WORKDIR app
ENV SQLX_OFFLINE='true'
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
COPY ./sqlx-data.json ./sqlx-data.json
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN mkdir -p data
RUN cargo build --release

FROM gcr.io/distroless/cc-debian10
COPY --from=builder /app/target/release/general_notifier /
COPY --from=builder /app/data /
COPY ./migrations ./migrations
COPY ./.env ./.env
CMD ["./general_notifier"]
