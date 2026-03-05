# End-to-End Test

This document describes the Docker-based end-to-end test for greptimedb-ros-agent. The test verifies the full data pipeline: ROS2 diagnostics messages are received by the agent and written to GreptimeDB.

## Prerequisites

- Docker and Docker Compose (v2)

No ROS2 or Rust installation is required on the host machine.

## Architecture

The test runs three Docker services on a shared network:

```
┌──────────────────┐       ROS2 DDS        ┌──────────────┐      gRPC       ┌─────────────┐
│  test-publisher  │ ───────────────────▶   │    agent     │ ─────────────▶  │  greptimedb  │
│  (ros:humble)    │   /diagnostics topic   │  (Dockerfile)│   port 4001    │  (standalone) │
│  1 Hz publish    │                        │  2s flush    │                │  port 4000    │
└──────────────────┘                        └──────────────┘                └──────────────┘
                                                                                  ▲
                                                                                  │ HTTP SQL
                                                                            ┌─────┴─────┐
                                                                            │  test.sh   │
                                                                            │  (host)    │
                                                                            └───────────┘
```

| Service | Image | Role |
|---------|-------|------|
| `greptimedb` | `greptime/greptimedb:latest` | Time-series database, standalone mode |
| `agent` | Built from `Dockerfile` | The greptimedb-ros-agent under test |
| `test-publisher` | `ros:humble` | Publishes fake `DiagnosticArray` messages |

## Test Data

The test publisher (`publish_test_diagnostics.py`) sends a `DiagnosticArray` at 1 Hz with one `DiagnosticStatus` entry:

| Field | Value |
|-------|-------|
| `name` | `Motor Driver` (creates table `motor_driver`) |
| `hardware_id` | `mc01` |
| `level` | `OK` (0) |
| `message` | `OK` |
| `temperature` | `42.5` |
| `voltage` | `24.1` |

## Running the Test

```bash
# 1. Start GreptimeDB (wait for healthy)
docker compose up -d greptimedb

# 2. Build and start agent + publisher
docker compose up -d --build agent test-publisher

# 3. Wait a few seconds for data to flow, then verify
bash test.sh

# 4. Clean up
docker compose down
```

Or as a one-liner:

```bash
docker compose up -d greptimedb && \
docker compose up -d --build agent test-publisher && \
bash test.sh; \
docker compose down
```

## What `test.sh` Does

The script polls GreptimeDB's HTTP SQL API, retrying up to 30 times (5-second intervals, 150 seconds total):

```
SELECT * FROM motor_driver ORDER BY ts DESC LIMIT 5
```

- **Success** (exit 0): HTTP 200 response containing `mc01` in the result body.
- **Failure** (exit 1): No matching data found after all retries.

## Files

| File | Purpose |
|------|---------|
| `Dockerfile` | Builds the agent on `ros:humble` with Rust and ROS2 dependencies |
| `docker-compose.yml` | Orchestrates greptimedb, agent, and test-publisher services |
| `publish_test_diagnostics.py` | Python ROS2 node that publishes test `DiagnosticArray` messages |
| `test.sh` | Verification script that queries GreptimeDB for expected data |

## Build Notes

- The `Dockerfile` uses `IDL_PACKAGE_FILTER="diagnostic_msgs;std_msgs;builtin_interfaces"` to limit r2r message generation, significantly reducing build time.
- The first build pulls the `ros:humble` image (~700 MB) and compiles all Rust dependencies. Subsequent builds use Docker layer caching and complete much faster.

## Troubleshooting

**Check service logs:**

```bash
docker compose logs greptimedb
docker compose logs agent
docker compose logs test-publisher
```

**Verify ROS2 topic communication** (from inside a container):

```bash
docker compose exec agent bash -c "source /opt/ros/humble/setup.bash && ros2 topic list"
```

**Query GreptimeDB manually:**

```bash
curl "http://localhost:4000/v1/sql" --data-urlencode "sql=SELECT * FROM motor_driver ORDER BY ts DESC LIMIT 5"
```
