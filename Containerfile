# ── Stage 1: Build ────────────────────────────────────────────────────────────
# Uses the official Rust image with the musl target for a fully static binary.
# No dynamic linking means the final image can be FROM scratch.
FROM rust:1-slim AS builder

RUN rustup target add x86_64-unknown-linux-musl \
    && apt-get update -qq \
    && apt-get install -y --no-install-recommends musl-tools \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ── Cache dependencies ────────────────────────────────────────────────────────
# Copy only the manifests first so dependency compilation is cached separately
# from source changes.
COPY Cargo.toml Cargo.lock ./

# Create dummy source files so cargo can resolve and compile the dependency
# graph once; the real source build below then only recompiles this crate.
RUN mkdir src \
    && echo 'fn main() {}' > src/main.rs \
    && echo '' > src/lib.rs \
    && cargo build --release --target x86_64-unknown-linux-musl 2>/dev/null || true \
    && rm -rf src

# ── Build the real source ─────────────────────────────────────────────────────
COPY . .

# Touch source files to invalidate the dummy build cache
RUN touch src/main.rs src/lib.rs

RUN cargo build --release --target x86_64-unknown-linux-musl

# ── Stage 2: Scratch ─────────────────────────────────────────────────────────
# Only the static binary plus a bundled story directory. The app makes no
# outbound network calls, so no CA certificates are needed.
FROM scratch

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/markdown-converter /markdown-converter
COPY test_story /story

EXPOSE 3000

ENTRYPOINT ["/markdown-converter"]
CMD ["/story"]