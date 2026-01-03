use super::atom_helper_filter;
use crate::proto::ClientConfig;
use regex::Regex;
use serde_json::Value;
use socketioxide::extract::SocketRef;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use tokio::time::Duration;

use crate::server::{RuntimeState, UiEvent};

pub fn extract_first_arg(value: Value) -> Value {
    match value {
        Value::Array(mut arr) => arr.pop().unwrap_or(Value::Null),
        other => other,
    }
}

pub fn ui_log(state: &RuntimeState, level: &'static str, message: impl Into<String>) {
    let _ = state.ui_tx.send(UiEvent::Log {
        level,
        message: message.into(),
    });
}

pub fn set_socket_if_empty(state: &RuntimeState, socket: SocketRef) -> bool {
    let mut guard = state.socket.lock().unwrap();
    if guard.is_some() {
        return false;
    }
    *guard = Some(socket);
    true
}

pub fn clear_socket(state: &RuntimeState) {
    *state.socket.lock().unwrap() = None;
}

pub fn reset_connection_state(state: &RuntimeState) {
    state.chunk_receive_state.lock().unwrap().clear();
    state.chunk_ack_waiters.lock().unwrap().clear();
    state.server_written.lock().unwrap().clear();
    state.client_written.lock().unwrap().clear();
    *state.client_config.lock().unwrap() = ClientConfig::default();
    *state.effective_regex.lock().unwrap() = state.server_side_regex.as_ref().clone();
}

pub fn rebuild_effective_regex(state: &RuntimeState, client: &ClientConfig) {
    let mut merged: Vec<Regex> = state.server_side_regex.as_ref().clone();
    if let Some(list) = &client.pathRegex {
        for s in list {
            if let Ok(re) = Regex::new(s) {
                merged.push(re);
            }
        }
    }
    *state.effective_regex.lock().unwrap() = merged;
}

pub fn merge_client_config(base: &mut ClientConfig, incoming: ClientConfig) {
    if incoming.enableFileSizeLimit.is_some() {
        base.enableFileSizeLimit = incoming.enableFileSizeLimit;
    }
    if incoming.maxFileSize.is_some() {
        base.maxFileSize = incoming.maxFileSize;
    }
    if incoming.pathRegex.is_some() {
        base.pathRegex = incoming.pathRegex;
    }
}

pub fn should_filter_rel(state: &RuntimeState, rel: &str) -> bool {
    let effective = state.effective_regex.lock().unwrap();
    atom_helper_filter::should_filter_path(rel, &effective)
}

pub fn mark_path_written(map: Arc<Mutex<HashMap<String, u64>>>, rel: &str, ttl: Duration) {
    let rel = rel.replace('\\', "/");
    let gen = {
        let mut guard = map.lock().unwrap();
        let next = guard.get(&rel).copied().unwrap_or(0).wrapping_add(1);
        guard.insert(rel.clone(), next);
        next
    };

    let map2 = map.clone();
    tokio::spawn(async move {
        tokio::time::sleep(ttl).await;
        let mut guard = map2.lock().unwrap();
        if guard.get(&rel).copied() == Some(gen) {
            guard.remove(&rel);
        }
    });
}

pub fn mark_client_written(state: &RuntimeState, rel: &str) {
    mark_path_written(state.client_written.clone(), rel, Duration::from_secs(12));
}

pub fn insert_ack_waiter(
    state: &RuntimeState,
    key: String,
    tx: oneshot::Sender<bool>,
) {
    state.chunk_ack_waiters.lock().unwrap().insert(key, tx);
}

pub fn remove_ack_waiter(state: &RuntimeState, key: &str) -> Option<oneshot::Sender<bool>> {
    state.chunk_ack_waiters.lock().unwrap().remove(key)
}
