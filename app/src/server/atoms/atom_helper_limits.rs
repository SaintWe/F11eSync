use crate::config::ServerConfig;
use crate::proto::ClientConfig;

fn kb(bytes: u64) -> String {
    format!("{:.1}KB", bytes as f64 / 1024.0)
}

pub fn validate_file_size(size: u64, client: &ClientConfig, cfg: &ServerConfig) -> Option<String> {
    let server_limit = cfg
        .enable_server_side_file_size_limit
        .then_some(cfg.max_server_side_file_size);

    let client_limit = client
        .enableFileSizeLimit
        .unwrap_or(false)
        .then_some(client.maxFileSize)
        .flatten();

    let effective_opt = match (server_limit, client_limit) {
        (None, None) => None,
        (Some(s), None) => Some(s),
        (None, Some(c)) => Some(c),
        (Some(s), Some(c)) => Some(s.min(c)),
    };
    let Some(effective) = effective_opt else {
        return None;
    };

    if size <= effective {
        return None;
    }

    let reason = match (server_limit, client_limit) {
        (Some(s), Some(c)) => format!(
            "文件过大 ({})，已跳过（服务端限制：{}，客户端限制：{}，生效：{}）",
            kb(size),
            kb(s),
            kb(c),
            kb(effective)
        ),
        (Some(s), None) => format!("文件过大 ({})，已跳过（服务端限制：{}）", kb(size), kb(s)),
        (None, Some(c)) => format!("文件过大 ({})，已跳过（客户端限制：{}）", kb(size), kb(c)),
        (None, None) => unreachable!(),
    };

    Some(reason)
}
