FROM rust:latest as build

WORKDIR /usr/src/task-streamer
COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc-debian10

COPY --from=build /usr/src/task-streamer/target/release/task-streamer /usr/local/bin/task-streamer

CMD ["task-streamer", "server"]
