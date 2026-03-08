# Use Alpine Linux as the base image
ARG ALPINE_VERSION="3.23"
FROM alpine:${ALPINE_VERSION}

# Install dependencies:
# - curl and ca-certificates are needed to download poof.
# - libgcc is needed by the riscv64 musl binary for stack unwinding.
RUN apk add --no-cache curl ca-certificates libgcc bash

# Download and install poof using the official script
RUN curl -fsSL https://raw.githubusercontent.com/pirafrank/poof/main/install.sh | sh

# The install script places poof in /root/.local/bin/poof by default.
# Set it to the PATH so it can be executed easily by any user context.
ENV PATH="/root/.local/bin:${PATH}"

# Set default shell
RUN sed -i 's|root:/bin/sh|root:/bin/bash|' /etc/passwd

# Enable poof for bash
RUN poof enable --shell bash

# Keep the container running in the background so you can use 'docker exec'
CMD ["poof", "version"]

# Example usage/alias setup:
# alias poof='docker run --rm -u $(id -u):$(id -g) -v ~/.local/share/poof:/.local/share/poof -e HOME=/ poof:latest poof'
