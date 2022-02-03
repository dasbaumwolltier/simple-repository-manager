FROM alpine:3.15

RUN apk --no-cache add ca-certificates

COPY target/x86_64-unknown-linux-musl/release/simple-repository-manager /usr/local/bin/simple-repository-manager
RUN chmod +x /usr/local/bin/simple-repository-manager

VOLUME ["/config.yaml"]

ENTRYPOINT /usr/local/bin/simple-repository-manager --config /config.yaml