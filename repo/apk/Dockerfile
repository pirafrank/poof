ARG ALPINE_VERSION=3.21
FROM alpine:${ALPINE_VERSION}

RUN apk add --no-cache abuild build-base

# Create a non-root packager user required by abuild
# abuild package already creates the abuild group, so only add the user
RUN adduser -S -G abuild -h /home/packager packager

# Allow the abuild group to install trusted keys so apk can verify signatures
RUN chgrp abuild /etc/apk/keys && chmod g+w /etc/apk/keys

COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

USER packager
WORKDIR /home/packager

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
