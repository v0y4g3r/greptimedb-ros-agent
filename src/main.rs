mod config;
mod convert;
mod subscriber;
mod writer;

use clap::Parser;
use config::Config;
use subscriber::{new_buffer, setup_subscriber};
use writer::GreptimeWriter;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = Config::parse();
    info!(?config, "Starting greptimedb-ros-agent");

    let buffer = new_buffer();
    let greptimedb = GreptimeWriter::new(&config.greptimedb_endpoint);

    // Set up ROS2 subscriber (spawns async task for message processing)
    let mut node = setup_subscriber(&config.topic, buffer.clone())?;

    // Spawn ROS2 spin loop on a dedicated thread
    let handle = tokio::task::spawn_blocking(move || {
        loop {
            node.spin_once(std::time::Duration::from_millis(100));
        }
    });

    // Flush loop: drain buffer and write to GreptimeDB at the configured interval
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(config.interval));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                let batch = {
                    let mut buf = buffer.lock().unwrap();
                    std::mem::take(&mut *buf)
                };
                greptimedb.write_batch(batch).await;
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Shutting down, flushing remaining buffer...");
                let batch = {
                    let mut buf = buffer.lock().unwrap();
                    std::mem::take(&mut *buf)
                };
                greptimedb.write_batch(batch).await;
                break;
            }
        }
    }

    handle.abort();
    info!("Shutdown complete");
    Ok(())
}
