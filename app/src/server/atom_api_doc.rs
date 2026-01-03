//! Atomic 4-Layer · Server Atom API Doc
//!
//! 目标：集中描述 server 侧 atoms/molecules 的“契约”（签名/输入输出/副作用/错误模型），不写实现。
//!
//! 分层说明：
//! - L2: `server/mod.rs`（启动/组装/生命周期）
//! - L3: `server/molecules/*`（业务动作：同步、上传处理、广播、Socket.IO 事件注册）
//! - L4: `server/atoms/*`（最小可信实现单元；其中 `atom_helper_*` 为纯计算 helper）
//!
//! ---------------------------------------------------------------------------
//! L3 · Molecules
//! ---------------------------------------------------------------------------
//! `molecules/socket_handlers.rs`
//! - `pub(crate) fn on_connect(socket: SocketRef, Data(data): Data<Value>, State(state): State<RuntimeState>)`
//!   - 副作用：注册 Socket.IO 事件、写日志、可能断开连接
//!   - 错误模型：对协议解码失败做忽略（不 panic），对业务失败写日志并 emit `sync_error`
//!
//! `molecules/sync_all.rs`
//! - `pub async fn run(state: &RuntimeState) -> Result<()>`
//!   - 含 IO：遍历目录、计算过滤、发送更新/分片
//!
//! `molecules/client_upload.rs`
//! - `handle_update/create_dir/chunk_*`：处理客户端上传与分片 ACK；含 IO（写文件/创建目录）
//! - `disconnect_cleanup`：断连时清理状态，停止分片重试/遍历等
//!
//! `molecules/fs_broadcast.rs`
//! - `handle_fs_event`：本地文件变化后广播给客户端（遵循过滤+大小限制）
//!
//! ---------------------------------------------------------------------------
//! L4 · Atoms
//! ---------------------------------------------------------------------------
//! `atoms/state.rs`
//! - `set_socket_if_empty`：设置单客户端 socket（拒绝第二个客户端）
//! - `merge_client_config`：合并 client config（只覆盖提供字段）
//! - `rebuild_effective_regex`：断连/重连时维护“服务端规则 + 客户端规则”合并结果
//!
//! `atoms/socket_emit.rs`
//! - `emit_*`：所有对客户端的 Socket.IO emit，副作用：网络发送
//!
//! `atoms/atom_helper_filter.rs`
//! - 纯计算：路径过滤匹配
//!
//! `atoms/atom_helper_limits.rs`
//! - 纯计算：服务端/客户端文件大小限制合并（取更小值）
//!
//! `atoms/atom_helper_messages.rs`
//! - 纯计算：日志/提示文案（不做 IO）

