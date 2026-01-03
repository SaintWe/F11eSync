//! Atomic 4-Layer · GUI Atom API Doc
//!
//! 目标：集中描述 GUI 侧 atoms/molecules/diplomat 的“契约”（签名/输入输出/副作用/错误模型），不写实现。
//!
//! 分层说明：
//! - L2: `commander.rs`（编排）、`data_officer.rs`（校验/归一化/结构转换）、`diplomat.rs`（外部交互封装）
//! - L3: `actions.rs`（业务动作）、`view.rs`/`molecules.rs`（视图组合）
//! - L4: `atoms.rs`/`styles.rs`/`atom_helper_*.rs`（最小可信实现/纯计算）
//!
//! ---------------------------------------------------------------------------
//! L2 · Diplomat（外部交互）
//! ---------------------------------------------------------------------------
//! `diplomat.rs`
//! - `pub fn pick_folder(current: &PathBuf) -> Option<PathBuf>`
//!   - 副作用：打开系统文件夹选择对话框
//!   - 错误：无（取消时返回 `None`）
//! - `pub fn save_log_file(current: &PathBuf) -> Option<PathBuf>`
//!   - 副作用：打开系统“保存文件”对话框
//!   - 错误：无（取消时返回 `None`）
//! - `pub fn ensure_dir_and_canonicalize(path: &PathBuf) -> anyhow::Result<PathBuf>`
//!   - 副作用：`create_dir_all` + `canonicalize`
//!   - 错误：文件系统错误
//! - `pub fn save_settings(app_cfg: &settings::AppConfig) -> anyhow::Result<()>`
//!   - 副作用：写入配置文件（YAML）
//!   - 错误：IO/序列化错误
//! - `pub fn init_tray() -> Option<TrayHandle>`
//!   - 副作用：创建系统托盘图标与菜单
//!   - 错误：资源缺失/创建失败返回 `None`
//! - `pub fn try_recv_tray_event_id() -> Option<String>`
//!   - 副作用：无（轮询 tray 事件队列）
//!   - 错误：无（无事件返回 `None`）
//!
//! ---------------------------------------------------------------------------
//! L2 · Data Officer（校验/归一化/结构转换）
//! ---------------------------------------------------------------------------
//! `data_officer.rs`
//! - `pub fn build_server_config(...) -> Option<ServerConfig>`
//!   - 输入：GUI 文本输入（host/port/max_size）+ 目录 + 规则编辑器内容
//!   - 输出：可运行的 `ServerConfig`（失败返回 `None`）
//!   - 副作用：无
//! - `pub fn build_app_config(...) -> settings::AppConfig`
//!   - 输入：当前 `ServerConfig` + UI 相关状态
//!   - 输出：用于落盘的 `settings::AppConfig`
//!   - 副作用：无
//!
//! ---------------------------------------------------------------------------
//! L3 · Actions（业务动作）
//! ---------------------------------------------------------------------------
//! `actions.rs`
//! - `start_stop`：启动/停止服务端线程，必要时展开日志侧边栏（窗口 resize）
//! - `browse_dir` / `apply_dir_input`：目录选择/应用（会触发写配置延迟保存）
//! - `check_update` / `download_update`：检查/下载更新（异步执行，结果回写日志）
//! - `copy_logs`：写剪贴板（副作用：剪贴板）
//! - `export_logs`：保存日志到文件（副作用：文件系统）
//!
//! ---------------------------------------------------------------------------
//! L4 · Helper Atoms（纯计算）
//! ---------------------------------------------------------------------------
//! `atom_helper_config.rs`
//! - `parse_server_side_path_regex(content: &text_editor::Content) -> Vec<String>`
//! - `parse_host_port(host: &str, port: &str) -> Option<(IpAddr, u16)>`
//! - `parse_max_server_side_file_size(max_size: &str) -> u64`
//!
//! `atom_helper_log.rs`
//! - 规范化日志行（去空/截断/统一格式），不做 IO

