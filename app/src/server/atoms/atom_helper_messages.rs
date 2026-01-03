pub fn format_ts_warning_line(reason: &str, title: &str) -> String {
    format!("{reason}: {title}")
}

pub fn format_chunk_progress(
    current: u32,
    total: u32,
    action: &str,
    is_zero_indexed: bool,
) -> Option<String> {
    if total == 0 {
        return None;
    }

    let completed = if is_zero_indexed { current.saturating_add(1) } else { current };
    let prev = if is_zero_indexed { current } else { current.saturating_sub(1) };
    let progress = (completed.saturating_mul(5)).div_ceil(total);
    let prev_progress = (prev.saturating_mul(5)).div_ceil(total);
    let is_last = if is_zero_indexed {
        current.saturating_add(1) >= total
    } else {
        current >= total
    };

    if progress > prev_progress || is_last {
        let pct = ((completed as f64 / total as f64) * 100.0).round() as u32;
        return Some(format!("{action}: {completed}/{total} ({pct}%)"));
    }
    None
}
