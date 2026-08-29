#!/usr/bin/env bash
# Build the bbox release binary inside a Docker container, so no Rust
# toolchain is needed on the host.
#
#   ./build-docker.sh
#
# Produces a statically-linked musl binary at target/release/bbox. This is
# required: Home Assistant's command_line integration runs bbox *inside* the
# HA container, which is Alpine/musl — a glibc build will not execute there.
#
# The build runs as root in the container (apk needs it), then hands the
# output files back to the invoking user. Override the image with RUST_IMAGE,
# e.g. RUST_IMAGE=rust:1.93-alpine ./build-docker.sh
#
# Deploy to the Home Assistant stack with:
#   sudo install -o root -g root target/release/bbox \
#     ~/ha-stack/ha-config/bbox-data/bbox
set -euo pipefail

cd "$(dirname "$0")"

RUST_IMAGE="${RUST_IMAGE:-rust:1-alpine}"

docker run --rm \
  -v "$PWD":/build \
  -w /build \
  -e CARGO_HOME=/build/.cargo-home \
  -e HOST_UID="$(id -u)" \
  -e HOST_GID="$(id -g)" \
  "$RUST_IMAGE" \
  sh -c 'apk add --no-cache build-base >/dev/null && \
         cargo build --release --locked && \
         chown -R "$HOST_UID:$HOST_GID" target .cargo-home'

echo
echo "Built: $PWD/target/release/bbox"
file target/release/bbox
