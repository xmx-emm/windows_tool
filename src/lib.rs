//! # Mx Tool
//! 用于Mx toolbox的一些rust后端库
//! 提供了设置window端,端口代理的方法

pub mod utils;

///游戏的内容
#[cfg(all(target_os = "windows", feature = "game"))]
pub mod game;

///elevated 只在windows下有效
#[cfg(all(target_os = "windows", feature = "elevated"))]
pub mod elevated;

///端口转发 只在windows下有效
#[cfg(all(target_os = "windows", feature = "port_forwarding"))]
pub mod port_forwarding;

///注册表
#[cfg(all(target_os = "windows", feature = "registry"))]
pub mod registry;

#[cfg(all(feature = "vdf"))]
pub mod vdf;
#[cfg(all(feature = "steam"))]
pub mod steam;

#[cfg(test)]
mod tests {
    use crate::elevated::is_elevated;
    use crate::port_forwarding::PortForwarding;
    use crate::registry::backups::{backups_explorer_registry, check_backups_explorer_registry};
    use crate::steam::get_steam_game_language;
    use crate::steam::user::{get_steam_users, get_steam_users_id};
    use crate::utils::{check_ipv4_by_string, run_multiple_commands, Println};

    #[test]
    fn test() {
        let p = PortForwarding::get_ipv4_to_ipv4();
        println!("ipv4_to_ipv4 {:?}", p);
    }

    #[test]
    fn port_forwarding() {
        assert!(check_ipv4_by_string(&"127.0.0.1".to_string()));
        assert!(!check_ipv4_by_string(&"asef.asef.0.1".to_string()));
        assert!(!check_ipv4_by_string(&"12.0.0".to_string()));
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
        let out = run_multiple_commands(&vl, true, true);
        out.print_ln()
    }

    #[test]
    fn elevated() {
        assert!(!is_elevated())
    }

    #[test]
    fn registry() {
        // assert_eq!(get_all_state().len(), 9);
        // get_all_state();
        assert!(backups_explorer_registry("mx_tools"));
        assert!(check_backups_explorer_registry("mx_tools"));
    }

    #[test]
    fn game() {
        let users = get_steam_users_id();
        println!("all steam users id {:?}", users);
        for user in users.unwrap() {
            let ui = user.parse::<usize>().unwrap();
            // let vdf = VdfValue::load_from_user_id(ui).unwrap();
            let a = get_steam_game_language(ui, 1172470);
            println!("language {:#?}", a);
        }
        println!("get_steam_users {:?}", get_steam_users());
    }

    #[test]
    fn hex() {
        let hex_string = "0055736572436f6e66696700016c616e677561676500736368696e657365000808";

        // 将16进制字符串转换为字节数组
        let bytes = hex::decode(hex_string).unwrap();

        // 尝试解析为字符串（UTF-8）
        println!("原始字节: {:?}", bytes);

        // 按null字节(0x00)分割字符串
        let parts: Vec<&[u8]> = bytes.split(|&b| b == 0x00).collect();

        for (i, part) in parts.iter().enumerate() {
            if !part.is_empty() {
                match std::str::from_utf8(part) {
                    Ok(s) => println!("部分 {}: '{}'", i, s),
                    Err(_) => println!("部分 {}: 非UTF8数据: {:?}", i, part),
                }
            }
        }

        // 整体转换为字符串（忽略无法转换的部分）
        let decoded = unsafe { String::from_utf8_unchecked(bytes) };
        println!("\n整体转换: {}", decoded);
        println!("encode\t{}", hex::encode(decoded));
    }
}
