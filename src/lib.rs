//! # windows_tool
//!
//! 面向 **Windows** 的实用库：提权检测、注册表与命令行辅助、基于 `netsh` 的 IPv4 端口代理、Steam VDF/启动项/语言，以及部分游戏（PUBG、Apex、EA Desktop）的封装。
//!
//! **平台**：除 [`crate::utils`] 外，各功能模块在 `target_os = "windows"` 下才可用；文档在 [docs.rs](https://docs.rs/windows_tool) 上针对 Windows 目标构建。若在非 Windows 上依赖本库，请关闭不需要的特性并注意编译条件。
//!
//! ## 特性（Cargo `features`）
//!
//! | 特性 | 说明 |
//! |------|------|
//! | `elevated` | 检测是否管理员提升、请求以管理员重启当前可执行文件 |
//! | `registry` | 注册表相关（含 Steam 安装路径等） |
//! | `port_forwarding` | `netsh interface portproxy` 封装（`serde` + `serde_json`，备份 JSON） |
//! | `vdf` | Steam KeyValues（VDF）解析与读写（依赖 `indexmap`、`winreg`） |
//! | `steam` | 在 `vdf` 之上读写 `localconfig`、库文件夹、启动项等（API 位于 [`game::steam`]） |
//! | `game` | 启用 `steam` 并包含 `game::steam` 以及 `game::pubg`、`game::apex`、`game::ea` 等 |
//!
//! **默认特性**：`elevated`、`registry`、`port_forwarding`、`game`。
//!
//! 关闭默认特性时，请自行启用所需模块；`elevated` 与 `registry` 会拉取 `encoding_rs`，以便命令行输出按 GBK 解码。`registry` 与 `port_forwarding` 还会拉取 `winapi`（`utils::filesystem` 等需要）。
//!
//! ## 模块概览
//!
//! - [`utils`]：IPv4 校验、时间格式、在管理员上下文中执行命令、控制台输出等（依赖 Windows 进程扩展）。
//! - [`elevated`]（`feature = "elevated"`）：[`elevated::is_elevated`]、[`elevated::request_restart_with_privileges_elevate`]。
//! - [`port_forwarding`]（`feature = "port_forwarding"`）：[`port_forwarding::PortForwarding`]。
//! - [`registry`]（`feature = "registry"`）：注册表备份、Steam 路径、Windows 更新暂停天数等。
//! - [`vdf`]（`feature = "vdf"`）：[`vdf::VdfValue`]、[`vdf::parse_vdf_string`]。
//! - [`game::steam`]（`feature = "steam"`）：启动项、语言、库路径、用户枚举等；**Steam 用户 ID 与游戏 App ID 均为 `usize`**。
//! - [`game`]（`feature = "steam"`）：根模块；**`feature = "game"`** 时额外包含 Apex / EA / PUBG 子模块。
//!
//! 更完整的依赖写法与示例见本仓库根目录的 `README.md`（与 [docs.rs 文档](https://docs.rs/windows_tool)）。

pub mod utils;

/// 游戏与 Steam 数据访问：`game::steam` 需 `feature = "steam"`；Apex / EA / PUBG 需 `feature = "game"`。
#[cfg(all(target_os = "windows", feature = "steam"))]
pub mod game;

/// 检测/请求管理员权限（仅 Windows）。
#[cfg(all(target_os = "windows", feature = "elevated"))]
pub mod elevated;

/// 基于 `netsh` 的 IPv4 端口转发（仅 Windows）。
#[cfg(all(target_os = "windows", feature = "port_forwarding"))]
pub mod port_forwarding;

/// 注册表读写与备份等（仅 Windows）。
#[cfg(all(target_os = "windows", feature = "registry"))]
pub mod registry;

/// Steam KeyValues（VDF）解析与修改。
#[cfg(all(feature = "vdf"))]
pub mod vdf;

#[cfg(all(test, target_os = "windows", feature = "port_forwarding"))]
mod tests_port_forwarding {
    use crate::port_forwarding::PortForwarding;
    use crate::utils::{check_ipv4_by_string, run_multiple_commands, Println, RunCommandOptions};

    #[test]
    fn list_port_proxy() {
        let p = PortForwarding::get_ipv4_to_ipv4();
        println!("ipv4_to_ipv4 {:?}", p);
    }

    #[test]
    fn port_forwarding_roundtrip() {
        assert!(check_ipv4_by_string("127.0.0.1"));
        assert!(!check_ipv4_by_string("asef.asef.0.1"));
        assert!(!check_ipv4_by_string("12.0.0"));
        PortForwarding::reset();
        let item = PortForwarding::new(
            ("127.0.0.1".to_string(), 100),
            ("127.0.0.1".to_string(), 666),
        );
        item.forward();
        let p = PortForwarding::get_ipv4_to_ipv4();
        println!("ipv4_to_ipv4 {:?}", p);
        println!("item.check {}", item.check());
    }

    #[test]
    fn multiple_cmd() {
        let vl = vec![
            "netsh interface portproxy add v4tov4 listenaddress=127.0.0.1 listenport=100  connectaddress=127.0.0.1 connectport=100",
            "netsh interface portproxy add v4tov4 listenaddress=127.0.0.1 listenport=100  connectaddress=127.0.0.1 connectport=106",
            "netsh interface portproxy add v4tov4 listenaddress=127.0.0.1 listenport=100  connectaddress=127.0.0.1 connectport=108",
            "netsh interface portproxy show v4tov4",
        ];
        let out = run_multiple_commands(&vl, RunCommandOptions::new(true, true, true));
        out.print_ln()
    }
}

#[cfg(all(test, target_os = "windows", feature = "elevated"))]
mod tests_elevated {
    use crate::elevated::is_elevated;

    #[test]
    fn elevated_state() {
        let _ = is_elevated();
    }
}

#[cfg(all(test, target_os = "windows", feature = "registry"))]
mod tests_registry {
    use crate::registry::backups::{backups_explorer_registry, check_backups_explorer_registry};

    #[test]
    fn explorer_registry_backup() {
        assert!(backups_explorer_registry("mx_tools"));
        assert!(check_backups_explorer_registry("mx_tools"));
    }
}

#[cfg(all(test, target_os = "windows", feature = "steam"))]
mod tests_steam {
    use crate::game::steam::get_steam_game_language;
    use crate::game::steam::user::{get_steam_users, get_steam_users_id};

    #[test]
    fn steam_users_and_language() {
        let users = get_steam_users_id();
        println!("all steam users id {:?}", users);
        if let Ok(users) = users {
            for user in users {
                let ui = user.parse::<usize>().unwrap();
                let a = get_steam_game_language(ui, 1172470);
                println!("language {:#?}", a);
            }
        }
        println!("get_steam_users {:?}", get_steam_users());
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests_hex_sample {
    #[test]
    fn decode_userconfig_hex_sample() {
        let hex_string = "0055736572436f6e66696700016c616e677561676500736368696e657365000808";
        let bytes = hex::decode(hex_string).unwrap();
        println!("原始字节: {:?}", bytes);
        let parts: Vec<&[u8]> = bytes.split(|&b| b == 0x00).collect();
        for (i, part) in parts.iter().enumerate() {
            if !part.is_empty() {
                match std::str::from_utf8(part) {
                    Ok(s) => println!("部分 {}: '{}'", i, s),
                    Err(_) => println!("部分 {}: 非UTF8数据: {:?}", i, part),
                }
            }
        }
        let decoded = unsafe { String::from_utf8_unchecked(bytes) };
        println!("\n整体转换: {}", decoded);
        println!("encode\t{}", hex::encode(decoded));
    }
}
