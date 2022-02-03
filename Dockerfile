FROM alpine:3.15

RUN apk --no-cache add ca-certificates

COPY target/x86_64-unknown-linux-musl/release/simple-repository-manager /usr/local/bin/simple-repository-manager
RUN chmod +x /usr/local/bin/simple-repository-manager

ENTRYPOINT /usr/local/bin/simple-repository-manager --config /config.yaml --host 0.0.0.0 --port 8080