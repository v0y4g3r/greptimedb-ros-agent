use greptimedb_ingester::api::v1::*;
use greptimedb_ingester::helpers::schema::{field, tag, timestamp};
use greptimedb_ingester::helpers::values::{f64_value, string_value, timestamp_millisecond_value};

/// Sanitize a string to be a valid GreptimeDB table or column name.
pub fn sanitize_name(name: &str) -> String {
    if name.is_empty() {
        return "_unknown".to_string();
    }
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{sanitized}")
    } else {
        sanitized
    }
}

/// Convert diagnostic level byte to human-readable string.
pub fn level_to_string(level: i8) -> &'static str {
    match level {
        0 => "OK",
        1 => "WARN",
        2 => "ERROR",
        3 => "STALE",
        _ => "UNKNOWN",
    }
}

/// Convert a DiagnosticStatus into a RowInsertRequest for GreptimeDB.
pub fn convert_status_to_insert_request(
    status: &r2r::diagnostic_msgs::msg::DiagnosticStatus,
    timestamp_ms: i64,
) -> RowInsertRequest {
    let table_name = sanitize_name(&status.name);

    let mut schema = vec![
        tag("hardware_id", ColumnDataType::String),
        tag("level", ColumnDataType::String),
        timestamp("ts", ColumnDataType::TimestampMillisecond),
        field("message", ColumnDataType::String),
    ];

    let mut values = vec![
        string_value(status.hardware_id.clone()),
        string_value(level_to_string(status.level).to_string()),
        timestamp_millisecond_value(timestamp_ms),
        string_value(status.message.clone()),
    ];

    for kv in &status.values {
        let col_name = sanitize_name(&kv.key);
        if let Ok(f) = kv.value.parse::<f64>() {
            schema.push(field(&col_name, ColumnDataType::Float64));
            values.push(f64_value(f));
        } else {
            schema.push(field(&col_name, ColumnDataType::String));
            values.push(string_value(kv.value.clone()));
        }
    }

    RowInsertRequest {
        table_name,
        rows: Some(Rows {
            schema,
            rows: vec![Row { values }],
        }),
    }
}

/// Convert a full DiagnosticArray message into a Vec of RowInsertRequests.
pub fn convert_diagnostic_array(
    msg: &r2r::diagnostic_msgs::msg::DiagnosticArray,
) -> Vec<RowInsertRequest> {
    let timestamp_ms = if msg.header.stamp.sec == 0 && msg.header.stamp.nanosec == 0 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    } else {
        (msg.header.stamp.sec as i64) * 1000 + (msg.header.stamp.nanosec as i64) / 1_000_000
    };

    msg.status
        .iter()
        .map(|s| convert_status_to_insert_request(s, timestamp_ms))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name_basic() {
        assert_eq!(sanitize_name("Motor Driver"), "motor_driver");
    }

    #[test]
    fn test_sanitize_name_slashes() {
        assert_eq!(sanitize_name("sensor/imu/data"), "sensor_imu_data");
    }

    #[test]
    fn test_sanitize_name_special_chars() {
        assert_eq!(sanitize_name("cpu-usage (%)"), "cpu_usage___");
    }

    #[test]
    fn test_sanitize_name_leading_number() {
        assert_eq!(sanitize_name("123sensor"), "_123sensor");
    }

    #[test]
    fn test_sanitize_name_empty() {
        assert_eq!(sanitize_name(""), "_unknown");
    }

    #[test]
    fn test_level_to_string() {
        assert_eq!(level_to_string(0), "OK");
        assert_eq!(level_to_string(1), "WARN");
        assert_eq!(level_to_string(2), "ERROR");
        assert_eq!(level_to_string(3), "STALE");
        assert_eq!(level_to_string(99), "UNKNOWN");
    }
}
