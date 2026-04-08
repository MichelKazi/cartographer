#!/bin/sh
set -e

REPO="michelkazi/cartographer"
INSTALL_DIR="/usr/local/bin"
BINARY="cartographer"

echo "installing cartographer..."

# macOS only
if [ "$(uname)" != "Darwin" ]; then
    echo "error: cartographer only runs on macOS" >&2
    exit 1
fi

# check arch
ARCH="$(uname -m)"
if [ "$ARCH" != "arm64" ]; then
    echo "error: only apple silicon (arm64) builds available right now" >&2
    echo "you'll need to build from source: cargo build --release" >&2
    exit 1
fi

# grab latest release
DOWNLOAD_URL="$(curl -sSf "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"browser_download_url"' \
    | grep 'cartographer' \
    | head -1 \
    | cut -d '"' -f 4)"

if [ -z "$DOWNLOAD_URL" ]; then
    echo "error: couldn't find a release to download" >&2
    exit 1
fi

# download to temp, move to install dir
TMP="$(mktemp)"
curl -sSfL "$DOWNLOAD_URL" -o "$TMP"
chmod +x "$TMP"

if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP" "${INSTALL_DIR}/${BINARY}"
else
    echo "need sudo to install to ${INSTALL_DIR}"
    sudo mv "$TMP" "${INSTALL_DIR}/${BINARY}"
fi

echo "installed to ${INSTALL_DIR}/${BINARY}"
echo ""
echo "run it with: cartographer"
echo "quit it with: pkill cartographer"
echo ""
echo "you'll need to grant accessibility permission on first run"
