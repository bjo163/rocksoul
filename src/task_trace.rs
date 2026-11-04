//! Append-only, content-free lifecycle records for approved workspace tasks.
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskTraceEvent {
    Proposed,
    Approved,
    Denied,
    Executed,
    Failed,
    Verified,
}

#[derive(Serialize)]
struct TaskTraceRecord<'a> {
    schema_version: u8,
    operation_id: &'a str,
    event: TaskTraceEvent,
    timestamp_unix_ms: u128,
    target: &'a str,
    detail: &'a str,
}

/// Appends and syncs one versioned JSONL record. No artifact body or secret is accepted.
pub fn append(
    path: &Path,
    operation_id: &str,
    event: TaskTraceEvent,
    target: &str,
    detail: &str,
) -> Result<()> {
    let timestamp_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?
        .as_millis();
    let record = TaskTraceRecord {
        schema_version: 1,
        operation_id,
        event,
        timestamp_unix_ms,
        target,
        detail,
    };
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("could not open task trace {}", path.display()))?;
    serde_json::to_writer(&mut file, &record).context("could not serialize task trace")?;
    file.write_all(b"\n")
        .context("could not finish task trace record")?;
    file.sync_data()
        .context("could not persist task trace record")?;
    Ok(())
}


/// Read the append-only JSONL trace for local inspection.
pub fn read(path: &Path) -> Result<Vec<serde_json::Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)
        .with_context(|| format!("could not read task trace {}", path.display()))?;
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str(line).context("task trace contains an invalid JSONL record")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{TaskTraceEvent, append};

    #[test]
    fn appends_versioned_content_free_jsonl_records() {
        let path = std::env::temp_dir().join(format!(
            "rocksoul-trace-{}.jsonl",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        append(
            &path,
            "workspace:note.txt",
            TaskTraceEvent::Proposed,
            "note.txt",
            "proposal",
        )
        .unwrap();
        append(
            &path,
            "workspace:note.txt",
            TaskTraceEvent::Approved,
            "note.txt",
            "approved",
        )
        .unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert_eq!(text.lines().count(), 2);
        assert!(text.contains("\"schema_version\":1"));
        assert!(!text.contains("private artifact body"));
        fs::remove_file(path).unwrap();
    }
}
