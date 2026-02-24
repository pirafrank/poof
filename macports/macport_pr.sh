#!/bin/bash

### Submit a PR to MacPorts official repo for a new version of poof.
### Run with: ./macport_pr.sh <portfile_path>

NAME="poof"
CATEGORY="sysutils"
PORTFILE_PATH="$1"
IS_GITHUB_ACTIONS=${GITHUB_ACTIONS:-false}

# Validate input
if [ -z "$PORTFILE_PATH" ]; then
    echo "❌ Error: Portfile path is required."
    echo "Usage: $0 <portfile_path>"
    exit 1
fi

if [ ! -f "$PORTFILE_PATH" ]; then
    echo "❌ Error: Portfile not found at '$PORTFILE_PATH'."
    exit 1
fi

VERSION=$(grep -E "^version\s+" "$PORTFILE_PATH" | sed -E 's/.*version\s+([0-9]+\.[0-9]+\.[0-9]+).*/\1/')

if [ -z "$VERSION" ]; then
    echo "❌ Error: Could not extract version from '$PORTFILE_PATH'."
    exit 1
fi

# Submit PR (interactive as script, automatic in GitHub Actions)
if [ "$IS_GITHUB_ACTIONS" != "true" ]; then
    read -p "Do you want to submit this to MacPorts official repo now? (y/n) " -n 1 -r
    echo
fi

if [[ $REPLY =~ ^[Yy]$ || "$IS_GITHUB_ACTIONS" == "true" ]]; then
    MP_DIR="/tmp/macports-ports"

    if [ ! -d "$MP_DIR" ]; then
        echo "Cloning macports-ports fork..."
        cd /tmp || exit 1
        # Note: idempotency: if fork already exists in user account,
        #       gh repo fork does not create a new one.
        #       it detects the existing fork and just clones it.
        gh repo fork macports/macports-ports --clone --remote
        mv /tmp/macports-ports "$MP_DIR"
    fi

    cd "$MP_DIR" || exit 1

    # Note: add upstream remote if not already present.
    #       It keeps fork's master branch up to date with
    #       the upstream (official repo) before creating a new branch.
    if ! git remote get-url upstream &>/dev/null; then
        git remote add upstream https://github.com/macports/macports-ports.git
    fi

    git checkout master && git pull upstream master
    git checkout -b "$NAME-$VERSION"

    if [ "$IS_GITHUB_ACTIONS" = "true" ]; then
        git config user.email "${GIT_USER_EMAIL:-actions@github.com}"
        git config user.name "${GIT_USER_NAME:-GitHub Actions}"
    fi

    mkdir -p "$CATEGORY/$NAME"
    cp "$PORTFILE_PATH" "$CATEGORY/$NAME/Portfile"

    git add "$CATEGORY/$NAME/Portfile"
    git commit -m "$NAME: update to $VERSION"
    git push origin "$NAME-$VERSION"

    gh pr create --title "$NAME: update to $VERSION" \
                 --body "Update $NAME to version $VERSION. Automatically generated via custom release script." \
                 --repo macports/macports-ports
fi
