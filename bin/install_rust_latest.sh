#!/bin/bash
set -e
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > /tmp/rustup.sh
sh /tmp/rustup.sh -y
~/.cargo/bin/rustc --version
~/.cargo/bin/cargo --version

