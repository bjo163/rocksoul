//! Append-only, content-free lifecycle records for approved workspace tasks.
use std::{
    fs::{self, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use fs2::FileExt;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskTraceEvent {
    Proposed,
    Approved,
    Denied,
    Executed,
    Failed,
    Cancelled,
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
    let mut encoded = serde_json::to_vec(&record).context("could not serialize task trace")?;
    encoded.push(b'\n');
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(path)
        .with_context(|| format!("could not open task trace {}", path.display()))?;

    // OS-level exclusive locking serializes writers across threads and processes.
    FileExt::lock_exclusive(&file)
        .with_context(|| format!("could not lock task trace {}", path.display()))?;
    let end = file.seek(SeekFrom::End(0))?;
    if end > 0 {
        file.seek(SeekFrom::End(-1))?;
        let mut last = [0_u8; 1];
        file.read_exact(&mut last)?;
        if last[0] != b'\n' {
            anyhow::bail!("task trace ends with an incomplete record; refusing to append");
        }
        file.seek(SeekFrom::End(0))?;
    }
    file.write_all(&encoded)
        .context("could not append complete task trace record")?;
    file.sync_data()
        .context("could not persist task trace record")?;
    FileExt::unlock(&file)
        .with_context(|| format!("could not unlock task trace {}", path.display()))?;
    Ok(())
}

/// Read the append-only JSONL trace for local inspection.
fn parse_records(content: &str) -> Result<Vec<serde_json::Value>> {
    let complete_end = content.rfind('\n').map_or(0, |index| index + 1);
    content[..complete_end]
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let record: serde_json::Value = serde_json::from_str(line)
                .context("task trace contains an invalid JSONL record")?;
            anyhow::ensure!(
                record
                    .get("schema_version")
                    .and_then(serde_json::Value::as_u64)
                    == Some(1),
                "task trace contains an unsupported schema version"
            );
            anyhow::ensure!(
                record
                    .get("operation_id")
                    .and_then(serde_json::Value::as_str)
                    .is_some()
                    && record
                        .get("event")
                        .and_then(serde_json::Value::as_str)
                        .is_some()
                    && record
                        .get("target")
                        .and_then(serde_json::Value::as_str)
                        .is_some(),
                "task trace record is missing required fields"
            );
            Ok(record)
        })
        .collect()
}

pub fn read(path: &Path) -> Result<Vec<serde_json::Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)
        .with_context(|| format!("could not read task trace {}", path.display()))?;
    parse_records(&content)
}

/// Read a bounded newest-first window for interactive inspectors.
/// The byte limit is enforced while reading, not only by a racy metadata pre-check.
pub fn read_recent(
    path: &Path,
    max_bytes: u64,
    max_records: usize,
) -> Result<Vec<serde_json::Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let byte_limit = max_bytes.min(1_048_576);
    let file = fs::File::open(path)
        .with_context(|| format!("could not open task trace {}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(byte_limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .with_context(|| "could not read bounded task trace")?;
    anyhow::ensure!(
        bytes.len() as u64 <= byte_limit,
        "task trace exceeds the interactive read limit"
    );
    let content = String::from_utf8(bytes).context("task trace is not valid UTF-8")?;
    let mut records = parse_records(&content)?;
    records.reverse();
    records.truncate(max_records.min(1_000));
    Ok(records)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{TaskTraceEvent, append, read, read_recent};

    #[test]
    fn read_recent_returns_newest_records_first_and_respects_limit() {
        let path = std::env::temp_dir().join(format!(
            "rocksoul-trace-recent-{}.jsonl",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for index in 0..3 {
            append(
                &path,
                &format!("op-{index}"),
                TaskTraceEvent::Proposed,
                "note.txt",
                "safe metadata",
            )
            .unwrap();
        }
        let records = read_recent(&path, 16_384, 2).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["operation_id"], "op-2");
        assert_eq!(records[1]["operation_id"], "op-1");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_recent_rejects_oversized_trace_before_reading() {
        let path = std::env::temp_dir().join(format!(
            "rocksoul-trace-large-{}.jsonl",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&path, vec![b'x'; 32]).unwrap();
        assert!(read_recent(&path, 16, 10).is_err());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn ignores_incomplete_trailing_record_and_refuses_to_append_after_it() {
        let path = std::env::temp_dir().join(format!(
            "rocksoul-trace-partial-{}.jsonl",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        append(
            &path,
            "op-1",
            TaskTraceEvent::Proposed,
            "note.txt",
            "proposal",
        )
        .unwrap();
        use std::io::Write as _;
        let mut file = fs::OpenOptions::new().append(true).open(&path).unwrap();
        file.write_all(b"{\"partial\":").unwrap();
        assert_eq!(read(&path).unwrap().len(), 1);
        assert!(append(&path, "op-2", TaskTraceEvent::Failed, "other.txt", "failed").is_err());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn append_helper_process() {
        if let (Ok(path), Ok(operation_id)) = (
            std::env::var("ROCKSOUL_TRACE_HELPER_PATH"),
            std::env::var("ROCKSOUL_TRACE_HELPER_OPERATION"),
        ) {
            append(
                std::path::Path::new(&path),
                &operation_id,
                TaskTraceEvent::Proposed,
                "process.txt",
                "cross-process proposal",
            )
            .unwrap();
        }
    }

    #[test]
    fn cross_process_appends_are_serialized_and_remain_parseable() {
        use std::process::Command;

        let path = std::env::temp_dir().join(format!(
            "rocksoul-trace-process-{}.jsonl",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let executable = std::env::current_exe().unwrap();
        let mut children = Vec::new();
        for index in 0..4 {
            children.push(
                Command::new(&executable)
                    .arg("--exact")
                    .arg("task_trace::tests::append_helper_process")
                    .arg("--nocapture")
                    .env("ROCKSOUL_TRACE_HELPER_PATH", &path)
                    .env(
                        "ROCKSOUL_TRACE_HELPER_OPERATION",
                        format!("process-{index}"),
                    )
                    .spawn()
                    .unwrap(),
            );
        }
        for mut child in children {
            assert!(child.wait().unwrap().success());
        }
        assert_eq!(read(&path).unwrap().len(), 4);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn concurrent_appends_are_serialized_and_remain_parseable() {
        let path = std::env::temp_dir().join(format!(
            "rocksoul-trace-concurrent-{}.jsonl",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut workers = Vec::new();
        for index in 0..8 {
            let path = path.clone();
            workers.push(std::thread::spawn(move || {
                append(
                    &path,
                    &format!("op-{index}"),
                    TaskTraceEvent::Proposed,
                    "note.txt",
                    "proposal",
                )
                .unwrap();
            }));
        }
        for worker in workers {
            worker.join().unwrap();
        }
        assert_eq!(read(&path).unwrap().len(), 8);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_unknown_schema_versions() {
        let path = std::env::temp_dir().join(format!(
            "rocksoul-trace-schema-{}.jsonl",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(
            &path,
            r#"{"schema_version":2,"operation_id":"op","event":"proposed","target":"x.txt"}
"#,
        )
        .unwrap();
        assert!(read(&path).is_err());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_malformed_complete_jsonl_records() {
        let path = std::env::temp_dir().join(format!(
            "rocksoul-trace-malformed-{}.jsonl",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&path, b"{not-json}\n").unwrap();
        assert!(read(&path).is_err());
        fs::remove_file(path).unwrap();
    }

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
        assert_eq!(read(&path).unwrap().len(), 2);
        assert!(text.contains("\"schema_version\":1"));
        assert!(!text.contains("private artifact body"));
        fs::remove_file(path).unwrap();
    }
}
