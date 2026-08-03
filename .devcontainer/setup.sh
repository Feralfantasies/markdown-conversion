#!/bin/bash
set -e # Exit immediately if a command fails

# 1. Setup Zsh
cp .devcontainer/zshrc-template ~/.zshrc

# 2. Setup Rust — stable toolchain + musl target for scratch container builds
rustup default stable
rustup target add x86_64-unknown-linux-musl

# 2b. Install sqlx-cli (rustls only — no openssl)
if ! command -v sqlx &> /dev/null; then
  cargo install sqlx-cli --no-default-features --features rustls,postgres
fi

# 3. Ensure the socket is accessible to the 'claude' user
# (VS Code mounts it as root:root by default)
sudo chown claude:claude /var/run/docker.sock || sudo chmod 666 /var/run/docker.sock

# 4. Add an alias so 'docker' commands in your Makefile call 'podman'
# (Since you mounted /usr/bin/podman but your Makefile likely says 'docker')
if ! command -v docker &> /dev/null; then
    sudo ln -s /usr/bin/podman /usr/bin/docker || alias docker=podman
fi
