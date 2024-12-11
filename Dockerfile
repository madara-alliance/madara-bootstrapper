# ==============================================
# Karnot Bootstrapper
# ==============================================
FROM ubuntu:22.04 AS builder

# Install basic dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    python3 \
    python3-pip \
    python3-venv \
    python3-dev \
    python3-cffi \
    libffi-dev \
    nodejs \
    npm \
    make \
    libgmp-dev \
    g++ \
    unzip \
    cmake \
    software-properties-common \
    && rm -rf /var/lib/apt/lists/*

# Install Python 3.7
RUN apt-get update && apt-get install -y software-properties-common && \
    add-apt-repository ppa:deadsnakes/ppa && \
    apt-get update && \
    apt-get install -y \
    python3.7 \
    python3.7-dev \
    python3.7-venv \
    python3.7-distutils \
    && rm -rf /var/lib/apt/lists/*

# Install pip for Python 3.7
RUN curl https://bootstrap.pypa.io/pip/3.7/get-pip.py -o get-pip.py && \
    python3.7 get-pip.py && \
    rm get-pip.py

# Install Python dependencies for legacy build
RUN python3.7 -m pip install --upgrade pip && \
    python3.7 -m pip install cmake==3.22

# Install specific solc version for legacy build
RUN curl https://binaries.soliditylang.org/linux-amd64/solc-linux-amd64-v0.6.12+commit.27d51765 -o /usr/local/bin/solc-0.6.12 && \
    echo 'f6cb519b01dabc61cab4c184a3db11aa591d18151e362fcae850e42cffdfb09a /usr/local/bin/solc-0.6.12' | sha256sum --check && \
    chmod +x /usr/local/bin/solc-0.6.12

# Setup Python virtual environment for main build
ENV VIRTUAL_ENV=/opt/venv
RUN python3 -m venv $VIRTUAL_ENV
ENV PATH="$VIRTUAL_ENV/bin:$PATH"

# Upgrade pip and install required Python packages
RUN python3 -m pip install --upgrade pip && \
    python3 -m pip install cffi && \
    python3 -m pip install solc-select && \
    solc-select install 0.8.19 && \
    solc-select use 0.8.19

# Install Foundry
SHELL ["/bin/bash", "-c"]
RUN curl -L https://foundry.paradigm.xyz | bash
ENV PATH="/root/.foundry/bin:${PATH}"
RUN source /root/.bashrc && foundryup

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install asdf and scarb
RUN git clone https://github.com/asdf-vm/asdf.git ~/.asdf --branch v0.13.1 && \
    echo '. "$HOME/.asdf/asdf.sh"' >> ~/.bashrc && \
    echo '. "$HOME/.asdf/completions/asdf.bash"' >> ~/.bashrc && \
    . "$HOME/.asdf/asdf.sh" && \
    asdf plugin add scarb && \
    asdf install scarb 2.8.4 && \
    asdf global scarb 2.8.4

# Set working directory
WORKDIR /app

# Copy the entire project
COPY . .

# Initialize and update submodules
RUN git submodule update --init --recursive

RUN apt-get update && apt-get install -y \
    wget

RUN curl https://binaries.soliditylang.org/linux-amd64/solc-linux-amd64-v0.6.12+commit.27d51765 -o /usr/local/bin/solc-0.6.12
RUN echo 'f6cb519b01dabc61cab4c184a3db11aa591d18151e362fcae850e42cffdfb09a /usr/local/bin/solc-0.6.12' | sha256sum --check
RUN chmod +x /usr/local/bin/solc-0.6.12
RUN npm install -g --unsafe-perm ganache-cli@6.12.2

# First run setup-linux
RUN make setup-linux

# Then build legacy contracts
RUN cd lib/starkgate-contracts-old && \
    # First verify ganache-cli installation
    which ganache-cli && \
    ganache-cli --version && \
    # Start ganache-cli in background with specific host and port
    nohup ganache-cli \
        --host 0.0.0.0 \
        --port 8545 \
        --networkId 1234 \
        --accounts 10 \
        --defaultBalanceEther 1000 \
        --mnemonic "test test test test test test test test test test test junk" \
        --db /tmp/ganache_db \
        > ganache.log 2>&1 & \
    # Store PID and wait
    GANACHE_PID=$! && \
    echo "Started Ganache with PID: $GANACHE_PID" && \
    sleep 15 && \
    # Debug: show ganache logs
    echo "Ganache logs:" && \
    cat ganache.log && \
    # Debug: check if process is running
    ps aux | grep ganache && \
    # Verify ganache is running
    curl -v http://localhost:8545 && \
    # Continue with build
    rm -rf build && \
    ./build.sh && \
    mkdir -p build/Release && \
    mkdir -p starkgate-artifacts && \
    cp -r build/Release/src/* starkgate-artifacts/ && \
    # Kill ganache after build
    kill $GANACHE_PID || true

RUN npm install -g --unsafe-perm ganache@7.9.0

# Generate other artifacts
RUN . "$HOME/.asdf/asdf.sh" && \
    make starkgate-contracts-latest && \
    make braavos-account-cairo && \
    make argent-contracts-starknet

# Make sure Ganache is installed right before build
# RUN npm install -g ganache

# Build the Rust project with specific binary name
RUN cargo build --release --workspace --bin karnot-bridge-deploy

# Runtime stage
FROM debian:buster-slim

# Copy only the compiled binary and artifacts
COPY --from=builder /app/target/release/karnot-bridge-deploy /usr/local/bin/
COPY --from=builder /app/artifacts /app/artifacts

# Set working directory
WORKDIR /app

# Environment variables
ENV RUST_LOG=info

# Run the binary
ENTRYPOINT ["/usr/local/bin/karnot-bridge-deploy"]