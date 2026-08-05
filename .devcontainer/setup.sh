#!/bin/bash
set -e # Exit immediately if a command fails

# 1. Setup Zsh
cat "$(dirname "$0")/zshrc-template" >>"$HOME/.zshrc"

# 2. Setup Rust — stable toolchain + musl target for scratch container builds
rustup default stable
rustup target add x86_64-unknown-linux-musl

# 2b. Install sqlx-cli (rustls only — no openssl)
if ! command -v sqlx &>/dev/null; then
	cargo install sqlx-cli --no-default-features --features rustls,postgres
fi

# 3. Ensure the socket is accessible to the 'claude' user
# (VS Code mounts it as root:root by default)
if [ -S /var/run/docker.sock ]; then
	sudo chown claude:claude /var/run/docker.sock ||
		sudo chgrp claude /var/run/docker.sock && sudo chmod 660 /var/run/docker.sock
else
	echo "WARNING: /var/run/docker.sock is not present; skipping container socket setup."
fi

# 4. Add an alias so 'docker' commands in your Makefile call 'podman'
# (Since you mounted /usr/bin/podman but your Makefile likely says 'docker')
if ! command -v docker &>/dev/null; then
	sudo ln -s /usr/bin/podman /usr/bin/docker || alias docker=podman
fi
