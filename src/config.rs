use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "greptimedb-ros-agent")]
#[command(about = "Bridges ROS2 diagnostics to GreptimeDB")]
pub struct Config {
    /// ROS2 topic to subscribe to
    #[arg(long, default_value = "/diagnostics")]
    pub topic: String,

    /// GreptimeDB gRPC endpoint (host:port)
    #[arg(long, default_value = "localhost:4001")]
    pub greptimedb_endpoint: String,

    /// Batch flush interval in seconds
    #[arg(long, default_value_t = 5)]
    pub interval: u64,
}
