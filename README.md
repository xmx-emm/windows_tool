# windows_tool

面向 **Windows** 的 Rust 库：管理员提权检测、注册表与命令行工具、IPv4 端口转发（`netsh portproxy`）、Steam VDF/启动项/语言，以及 PUBG、Apex、EA Desktop 等游戏的辅助封装。

在线 API 文档：<https://docs.rs/windows_tool>

## 添加依赖

在 `Cargo.toml` 中：

```toml
[dependencies]
windows_tool = "0.0.8"
```

仅启用部分功能时关闭默认特性，再按需打开：

```toml
[dependencies]
windows_tool = { version = "0.0.8", default-features = false, features = ["elevated", "registry"] }
```

## 特性（features）

| 特性 | 说明 |
|------|------|
| `elevated` | 是否已提升管理员、`request_restart_with_privileges_elevate` |
| `registry` | 注册表、Steam 安装路径等 |
| `port_forwarding` | `PortForwarding`（`serde`） |
| `vdf` | VDF 解析/读写（`indexmap`、`winreg`） |
| `steam` | 在 `vdf` 上读写 `localconfig`、库文件夹、启动项等 |
| `game` | 启用 `steam` 并包含 `game::*` 子模块 |

默认：`elevated`、`registry`、`port_forwarding`、`game`。

单独启用 `registry` 或 `port_forwarding` 时也会带上 `winapi`（`utils::path` 依赖）；`elevated` 与 `registry` 会带上 `encoding_rs`（命令输出 GBK 解码）。

**约定**：Steam **用户 ID** 与 **游戏 App ID** 在公开 API 中均使用 `usize`（与 Steam 目录名、App ID 数字一致）。

## 端口转发（`port_forwarding`）

基于 `netsh interface portproxy`。**添加、删除、重置**一般需要管理员；**仅查询列表**通常不需要。

```rust
use windows_tool::port_forwarding::PortForwarding;

// 列出当前 IPv4→IPv4 规则
let rules = PortForwarding::get_ipv4_to_ipv4();

// 新建一条：监听 127.0.0.1:100，转发到 127.0.0.1:666
let item = PortForwarding::new(("127.0.0.1", 100), ("127.0.0.1", 666));
item.forward();

// 批量（内部会过滤非法 IPv4）
PortForwarding::new_multiple(vec![item]);

// 清空所有规则
PortForwarding::reset();
```

在 PowerShell 里以管理员执行多条 `netsh` 的等价写法示例（仅供参考，库内已封装命令执行）：

```powershell
Start-Process cmd "/c netsh interface portproxy add v4tov4 listenport=4000 listenaddress=10.0.0.113 connectaddress=192.168.21.4 connectport=22" -Verb RunAs
```

## 提权（`elevated`）

```rust
use windows_tool::elevated::{is_elevated, request_restart_with_privileges_elevate};

if !is_elevated() {
    // 部分操作需要管理员
}

// 以管理员重启当前 exe（可能影响拖拽文件到控制台窗口等行为）
request_restart_with_privileges_elevate(false, true);
```

## 注册表（`registry`）

包含资源管理器相关项备份、Steam 路径查询、`modify_windows_update_flight_settings_max_pause_days` 等，详见 `windows_tool::registry` 模块文档。

## Steam 与 VDF（`game::steam` / `vdf`）

```rust
use windows_tool::game::steam::{
    get_steam_game_launch_options, set_steam_game_launch_options,
    get_steam_game_language,
};
// steam_user_id、steam_game_id 均为 usize
let opts = get_steam_game_launch_options(123456789, 578080)?;
let lang = get_steam_game_language(123456789, 578080)?;
set_steam_game_launch_options(123456789, 578080, "+exec autoexec.cfg")?;
```

解析任意 VDF 文本可使用 `windows_tool::vdf::parse_vdf_string`。

## 游戏子模块（`game`）

- `game::steam`（需 `feature = "steam"`）：`localconfig`、启动项、库路径、用户枚举等  
- `game::pubg`（需 `feature = "game"`）：PUBG（App ID `578080`）启动项读写  
- `game::apex`（需 `feature = "game"`）：Apex 启动项、语音 depot、安装路径等  
- `game::ea`（需 `feature = "game"`）：EA Desktop `user_*.ini`、Apex 启动命令行键  

## 许可证

MIT（见 `Cargo.toml` 中的 `license` 字段）。
