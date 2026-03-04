use std::sync::{Arc, Mutex};

use greptimedb_ingester::api::v1::RowInsertRequest;
use r2r::QosProfile;
use futures::StreamExt;
use tracing::{info, error};

use crate::convert::convert_diagnostic_array;

/// Buffer type shared between subscriber and flush loop.
pub type Buffer = Arc<Mutex<Vec<RowInsertRequest>>>;

/// Create a new shared buffer.
pub fn new_buffer() -> Buffer {
    Arc::new(Mutex::new(Vec::new()))
}

/// Set up the ROS2 node and subscribe to the diagnostics topic.
/// Returns the node (caller must spin it) and spawns the subscription
/// processing onto the tokio runtime.
pub fn setup_subscriber(
    topic: &str,
    buffer: Buffer,
) -> Result<r2r::Node, Box<dyn std::error::Error>> {
    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, "greptimedb_ros_agent", "")?;

    let mut sub = node.subscribe::<r2r::diagnostic_msgs::msg::DiagnosticArray>(
        topic,
        QosProfile::default(),
    )?;

    info!(topic, "Subscribed to ROS2 topic");

    tokio::spawn(async move {
        while let Some(msg) = sub.next().await {
            let inserts = convert_diagnostic_array(&msg);
            if let Ok(mut buf) = buffer.lock() {
                buf.extend(inserts);
            }
        }
    });

    Ok(node)
}
