FROM ros:humble

# Install build dependencies
RUN apt-get update && apt-get install -y \
    libclang-dev \
    build-essential \
    cmake \
    curl \
    pkg-config \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

# Limit r2r message generation to only what we need (much faster build)
ENV IDL_PACKAGE_FILTER="diagnostic_msgs;std_msgs;builtin_interfaces"

# Build with ROS2 environment sourced
SHELL ["/bin/bash", "-c"]
RUN source /opt/ros/humble/setup.bash && cargo build --release

# Entrypoint: source ROS2 and run the agent
ENTRYPOINT ["/bin/bash", "-c", "source /opt/ros/humble/setup.bash && exec /app/target/release/greptimedb-ros-agent \"$@\"", "--"]
