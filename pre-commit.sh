#!/bin/sh
echo "Format"
cargo fmt --all -- --check
echo "Clippy"
cargo clippy -- -D warnings
