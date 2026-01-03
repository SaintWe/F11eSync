use iced::widget::text_editor;
use std::net::IpAddr;

pub fn parse_server_side_path_regex(content: &text_editor::Content) -> Vec<String> {
    content
        .text()
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
}

pub fn parse_host_port(host: &str, port: &str) -> Option<(IpAddr, u16)> {
    let host: IpAddr = host.parse().ok()?;
    let port: u16 = port.parse().ok()?;
    Some((host, port))
}

pub fn parse_max_server_side_file_size(max_size: &str) -> u64 {
    max_size.parse().ok().unwrap_or(250 * 1024)
}
