FROM clux/muslrust as cargo-build
WORKDIR /usr/src/diamond
COPY . /usr/src/diamond
RUN apt-get update && apt-get install -y -y librrd-dev rrdtool
RUN cargo build --release
RUN rm /usr/src/diamond/target/x86_64-unknown-linux-musl/release/*.d

FROM alpine:latest
COPY --from=cargo-build  /usr/src/diamond/target/x86_64-unknown-linux-musl/release/diamond* /usr/bin/
COPY --from=cargo-build  /usr/src/diamond/target/x86_64-unknown-linux-musl/release/whisper* /usr/bin/
COPY --from=cargo-build  /usr/src/diamond/target/x86_64-unknown-linux-musl/release/find-corrupt-whisper-files /usr/bin/
CMD ["diamond-server"]
