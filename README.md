# GreptimeDB ROS Agent

A Rust agent that bridges ROS2 diagnostic messages to GreptimeDB time-series database.

## Overview

This agent subscribes to ROS2 `/diagnostics` topics and writes the diagnostic data to GreptimeDB for long-term storage and analysis. It's designed to run alongside ROS2 systems (e.g., robots, autonomous vehicles) to capture health and status information over time.

## Features

- Subscribes to ROS2 `diagnostic_msgs/msg/DiagnosticArray` messages
- Converts diagnostic status to time-series records in GreptimeDB
- Configurable flush interval for batched writes
- Automatic table creation based on diagnostic names
- Graceful shutdown with buffer flush
- Docker support for easy deployment

## Architecture

```
ROS2 /diagnostics → Agent (Rust) → GreptimeDB
```

## Deployment

```
┌─────────────────────────────────────────────────────────────────┐
│                         Ubuntu Host                             │
│                                                                 │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                     ROS2 Humble                            │ │
│  │  ┌──────────────┐    ┌──────────────────────────────────┐  │ │
│  │  │   Sensors    │───▶│  /diagnostics (DiagnosticArray)  │  │ │
│  │  │  Actuators   │    └──────────────┬───────────────────┘  │ │
│  │  │   Drivers    │                   │                      │ │
│  │  └──────────────┘                   │                      │ │
│  └─────────────────────────────────────┼──────────────────────┘ │
│                                        │                        │
│                                        ▼                        │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │              GreptimeDB ROS Agent (Rust)                   │ │
│  │                                                            │ │
│  │  • Subscribe to /diagnostics                               │ │
│  │  • Convert to time-series records                          │ │
│  │  • Batch & flush to GreptimeDB                             │ │
│  └─────────────────────────────────────┬──────────────────────┘ │
│                                        │                        │
│                                        │ gRPC (port 4001)       │
│                                        ▼                        │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                    GreptimeDB                              │ │
│  │                                                            │ │
│  │  Ports: 4000 (HTTP), 4001 (gRPC), 4002 (MySQL)             │ │
│  │  Schema-less time-series storage                           │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

The agent:
1. Subscribes to the configured ROS2 topic
2. Converts each `DiagnosticStatus` to a row insert request
3. Buffers inserts in memory
4. Flushes the buffer to GreptimeDB at the configured interval

Each unique diagnostic name becomes a separate table in GreptimeDB with:
- **Tags**: `hardware_id`, `level`
- **Timestamp**: `ts`
- **Fields**: `message`, plus any key-value pairs from the diagnostic

## Quick Start

### Using Docker Compose (Recommended)

```bash
docker compose up
```

This starts:
- **GreptimeDB** on ports 4000 (HTTP), 4001 (gRPC), 4002 (MySQL)
- **Agent** configured to connect to GreptimeDB
- **Test publisher** that publishes sample diagnostic data

### Verify Data

```bash
# Query GreptimeDB for motor driver diagnostics
curl -X POST "http://localhost:4000/v1/sql" \
  --data-urlencode "sql=SELECT * FROM motor_driver ORDER BY ts DESC LIMIT 10"
```

Or run the included test script:

```bash
./test.sh
```

## Configuration

The agent is configured via command-line arguments:

| Argument | Default | Description |
|----------|---------|-------------|
| `--topic` | `/diagnostics` | ROS2 topic to subscribe to |
| `--greptimedb-endpoint` | `localhost:4001` | GreptimeDB gRPC endpoint (host:port) |
| `--interval` | `5` | Batch flush interval in seconds |

### Example

```bash
greptimedb-ros-agent \
  --topic /robot/diagnostics \
  --greptimedb-endpoint 192.168.1.100:4001 \
  --interval 10
```

## Building

### Prerequisites

- Rust 1.85+ (edition 2024)
- ROS2 Humble (or compatible)
- `libclang-dev` for r2r bindings

### Build

```bash
# Source ROS2 environment first
source /opt/ros/humble/setup.bash

# Build
cargo build --release
```

### Docker Build

```bash
docker build -t greptimedb-ros-agent .
```

## Development

### Run Tests

```bash
cargo test
```

### Project Structure

```
src/
├── main.rs       # Entry point, orchestration
├── config.rs     # CLI configuration via clap
├── subscriber.rs # ROS2 subscription setup
├── convert.rs    # Diagnostic → GreptimeDB conversion
└── writer.rs     # GreptimeDB batch writer
```

## Data Schema

GreptimeDB is schema-less, so you don't need to manually create tables beforehand. Tables are automatically created and altered upon insertion as new diagnostics arrive.

For a diagnostic named `"Motor Driver"` with hardware_id `"mc01"`:

```sql
CREATE TABLE motor_driver (
  hardware_id STRING TAG,
  level STRING TAG,
  ts TIMESTAMP TIME INDEX,
  message STRING,
  temperature DOUBLE,
  voltage DOUBLE,
  ...
);
```

The schema is dynamic - additional key-value pairs in the diagnostic are added as columns.

## License

[Apache License 2.0](https://apache.org/licenses/LICENSE-2.0.txt)
