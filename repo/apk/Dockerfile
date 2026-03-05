ARG ALPINE_VERSION=3.21
FROM alpine:${ALPINE_VERSION}

RUN apk add --no-cache abuild

# Create a non-root packager user required by abuild
RUN addgroup -S abuild && \
    adduser -S -G abuild -h /home/packager packager

COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

USER packager
WORKDIR /home/packager

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
