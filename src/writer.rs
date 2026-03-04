use greptimedb_ingester::api::v1::*;
use greptimedb_ingester::client::Client;
use greptimedb_ingester::database::Database;
use tracing::{info, warn};

pub struct GreptimeWriter {
    database: Database,
}

impl GreptimeWriter {
    pub fn new(endpoint: &str) -> Self {
        let client = Client::with_urls(&[endpoint]);
        let database = Database::new_with_dbname("public", client);
        info!(endpoint, "Connected to GreptimeDB");
        Self { database }
    }

    /// Write a batch of insert requests to GreptimeDB.
    /// On failure, logs a warning and drops the batch.
    pub async fn write_batch(&self, inserts: Vec<RowInsertRequest>) {
        if inserts.is_empty() {
            return;
        }

        let request = RowInsertRequests { inserts };
        match self.database.insert(request).await {
            Ok(affected) => {
                info!(affected, "Batch written to GreptimeDB");
            }
            Err(e) => {
                warn!(error = %e, "Failed to write batch to GreptimeDB, dropping");
            }
        }
    }
}
