#!/bin/bash
# full release pipeline: version bump -> build -> GH release -> homebrew tap update
#
# usage:
#   ./scripts/release.sh patch    # 0.0.2 -> 0.0.3
#   ./scripts/release.sh minor    # 0.0.2 -> 0.1.0
#   ./scripts/release.sh major    # 0.0.2 -> 1.0.0

set -euo pipefail

LEVEL="${1:?usage: ./scripts/release.sh <patch|minor|major>}"
REPO="MichelKazi/cartographer"
TAP_REPO="MichelKazi/homebrew-tap"
BINARY_NAME="cartographer"

# grab the current version before bumping
PREV_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
echo "==> current version: ${PREV_VERSION}"

# 1. build release binary first so we don't push a broken tag
echo "==> building release binary..."
cargo build --release
BINARY="target/release/${BINARY_NAME}"

if [[ ! -f "$BINARY" ]]; then
    echo "!!! binary not found at ${BINARY}"
    exit 1
fi

# 2. cargo-release: bump version, commit, tag, push
echo "==> running cargo release ${LEVEL}..."
cargo release "${LEVEL}" --execute --no-confirm

# 3. read the new version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
TAG="v${VERSION}"
echo "==> released: ${PREV_VERSION} -> ${VERSION} (${TAG})"

# 4. rebuild with the new version in case it matters
cargo build --release

# 5. create github release with the binary
echo "==> creating github release ${TAG}..."
gh release create "${TAG}" "${BINARY}" \
    --repo "${REPO}" \
    --title "${TAG}" \
    --generate-notes

# 6. compute sha256 for homebrew
SHA=$(shasum -a 256 "${BINARY}" | awk '{print $1}')
echo "==> sha256: ${SHA}"

# 7. update homebrew tap
echo "==> updating homebrew tap..."
TAP_DIR=$(mktemp -d)
trap 'rm -rf "${TAP_DIR}"' EXIT

gh repo clone "${TAP_REPO}" "${TAP_DIR}" -- --depth 1 2>/dev/null

FORMULA="${TAP_DIR}/Formula/${BINARY_NAME}.rb"

if [[ ! -f "$FORMULA" ]]; then
    echo "!!! formula not found at ${FORMULA}"
    exit 1
fi

# update version, url, and sha256 in the formula
sed -i '' "s/version \"${PREV_VERSION}\"/version \"${VERSION}\"/" "${FORMULA}"
sed -i '' "s|/v${PREV_VERSION}/|/v${VERSION}/|" "${FORMULA}"

OLD_SHA=$(grep 'sha256' "${FORMULA}" | head -1 | awk -F'"' '{print $2}')
sed -i '' "s/${OLD_SHA}/${SHA}/" "${FORMULA}"

echo "==> formula diff:"
cd "${TAP_DIR}"
git diff

# commit and push
git add Formula/${BINARY_NAME}.rb
git commit -m "${BINARY_NAME} ${TAG}"
git push origin main

echo ""
echo "==> done. ${TAG} released."
echo "    github: https://github.com/${REPO}/releases/tag/${TAG}"
echo "    brew:   brew upgrade ${BINARY_NAME} (after: brew update)"
