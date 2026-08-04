FROM rust:1-bookworm

ARG TZ
ENV TZ="$TZ"
ARG GIT_DELTA_VERSION=0.18.2
ARG ZSH_IN_DOCKER_VERSION=1.2.0

# 1. INSTALL SYSTEM TOOLS FIRST
# We need 'sudo' to create the directory /etc/sudoers.d
# We need 'zsh' because we are assigning it as the default shell
# Note: Node.js is kept for MCP servers / tooling run via npx
RUN apt-get update && apt-get install -y --no-install-recommends \
  curl wget git sudo zsh vim nano jq procps aggregate podman uidmap \
  less fzf man-db unzip gnupg2 gh iptables ipset iproute2 dnsutils \
  musl-tools \
  && curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
  && apt-get install -y nodejs \
  && apt-get clean && rm -rf /var/lib/apt/lists/*

# 2. NOW CREATE THE USER
# Now that /etc/sudoers.d exists and /usr/bin/zsh exists, this will work.
RUN groupadd --gid 1000 claude \
  && useradd --uid 1000 --gid 1000 -m -s /usr/bin/zsh claude \
  && printf '%s\n' \
       "claude ALL=(root) NOPASSWD: /bin/chown claude\\:claude /var/run/docker.sock" \
       "claude ALL=(root) NOPASSWD: /bin/chmod 660 /var/run/docker.sock" \
       "claude ALL=(root) NOPASSWD: /bin/ln -s /usr/bin/podman /usr/bin/docker" \
       > /etc/sudoers.d/claude \
  && chmod 0440 /etc/sudoers.d/claude

# 3. SET UP PATHS & PERMISSIONS
ENV CARGO_HOME=/home/claude/.cargo
ENV RUSTUP_HOME=/home/claude/.rustup
ENV PATH="/home/claude/.cargo/bin:${PATH}"

RUN mkdir -p /workspace /commandhistory $CARGO_HOME $RUSTUP_HOME \
  && touch /commandhistory/.bash_history \
  && chown -R claude:claude /workspace /commandhistory /home/claude

# 4. INSTALL TOOLS
RUN ARCH=$(dpkg --print-architecture) \
  && wget "https://github.com/dandavison/delta/releases/download/${GIT_DELTA_VERSION}/git-delta_${GIT_DELTA_VERSION}_${ARCH}.deb" \
  && dpkg -i "git-delta_${GIT_DELTA_VERSION}_${ARCH}.deb" \
  && rm "git-delta_${GIT_DELTA_VERSION}_${ARCH}.deb"

# 5. ZSH & FIREWALL
RUN sh -c "$(wget -O- https://github.com/deluan/zsh-in-docker/releases/download/v${ZSH_IN_DOCKER_VERSION}/zsh-in-docker.sh)" -- \
  -p git -p fzf -u claude \
  -a "export PROMPT_COMMAND='history -a' && export HISTFILE=/commandhistory/.bash_history"

COPY init-firewall.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/init-firewall.sh \
  && echo "claude ALL=(root) NOPASSWD: /usr/local/bin/init-firewall.sh" > /etc/sudoers.d/claude-firewall

WORKDIR /workspace
USER claude