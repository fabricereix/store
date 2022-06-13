#!/bin/bash
set -eu
os="$1"

cargo build --release
strip target/release/store

package_dir="target/archive/store-$VERSION"
mkdir -p "$package_dir"
cp target/release/store "$package_dir"

upload_dir="target/upload"
mkdir -p "$upload_dir"
tarball_file="store-$VERSION-x86_64-$os.tar.gz"

tar cvfz "target/upload/$tarball_file" -C "$(dirname "$package_dir")" "store-$VERSION"
