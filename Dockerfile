FROM dasbaumwolltier/rust-musl-builder as build

COPY --chown=rust:rust . /build
WORKDIR /build

RUN ls -la /build &&\
    cargo build --release

RUN ls -la /build/target

FROM alpine:3.15

RUN apk update &&\
    apk upgrade &&\
    apk --no-cache add ca-certificates

COPY --from=build /build/target/x86_64-unknown-linux-musl/release/simple-repository-manager /usr/local/bin/simple-repository-manager
RUN chmod +x /usr/local/bin/simple-repository-manager

VOLUME ["/config.yaml"]

ENTRYPOINT /usr/local/bin/simple-repository-manager --config /config.yaml