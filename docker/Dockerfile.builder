FROM docker.io/backpackapp/build:v0.31.0

# Install Solana CLI
RUN sh -c "$(curl -sSfL https://release.solana.com/v1.17.17/install)"
ENV PATH="/root/.local/share/solana/install/active_release/bin:$PATH"

# Install Anchor CLI
RUN cargo install --locked anchor-cli --version 0.31.0

WORKDIR /workspace

# Simple build command - PATH is already set by ENV
CMD ["anchor", "build"]