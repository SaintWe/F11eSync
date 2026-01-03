#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Other,
}

use super::atom_helper_path;

pub fn classify_log_line(line: &str) -> LogLevel {
    let l = line.trim_start();
    if l.starts_with("[error]") || l.starts_with("[fatal]") {
        return LogLevel::Error;
    }
    if l.starts_with("[warn]") || l.starts_with("[warning]") {
        return LogLevel::Warn;
    }
    if l.starts_with("[info]") {
        return LogLevel::Info;
    }
    if l.starts_with("[debug]") || l.starts_with("[trace]") {
        return LogLevel::Debug;
    }
    LogLevel::Other
}

pub fn normalize_log_line(line: impl Into<String>) -> Option<String> {
    let mut line = line.into();
    if line.trim().is_empty() {
        return None;
    }
    if line.len() > 2000 {
        line.truncate(2000);
        line.push_str("…");
    }
    if let std::borrow::Cow::Owned(v) = atom_helper_path::normalize_windows_path_prefixes_in_text(&line) {
        line = v;
    }
    Some(line)
}
