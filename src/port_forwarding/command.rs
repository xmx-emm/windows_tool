use crate::port_forwarding::PortForwarding;
use crate::port_forwarding::cmd::{PORT_PROXY_V4TOV4, RESET_CMD, add_cmd, del_cmd, set_cmd};
use crate::utils::{Println, check_ipv4_by_string, run_commands, RunCommandOptions};
use ansi_term::Color::Red;

fn run_portproxy(cmd: &str) -> Result<(), String> {
    let out = run_commands(cmd, RunCommandOptions::new(true, true, true))?;
    out.print_ln();
    Ok(())
}

///
/// netsh interface portproxy add v4tov4 listenport=40005 listenaddress=10.0.0.113 connectaddress=192.168.21.4 connectport=22
///
pub fn add(forward: &PortForwarding) -> Result<(), String> {
    run_portproxy(&add_cmd(forward))
}

/// 删除一条项目(需要管理员权限)
pub fn del(address: &String, port: &i64) -> Result<(), String> {
    run_portproxy(&del_cmd(address, port))
}

/// 重置(需要管理员权限)
pub fn reset() -> Result<(), String> {
    run_portproxy(RESET_CMD)
}

///设置代理
pub fn set(port: &PortForwarding) -> Result<(), String> {
    run_portproxy(&set_cmd(port))
}

/// 获取所有的ipv4 to ipv4代理
pub fn get_all_ipv4_to_ipv4_port_proxy() -> Vec<PortForwarding> {
    match run_commands(PORT_PROXY_V4TOV4, RunCommandOptions::new(true, false, true)) {
        Ok(out) => {
            out.print_ln();
            from_cmd_load_port_forwarding(&out.to_string())
        }
        Err(e) => {
            println!("get_all_ipv4_to_ipv4_port_proxy failed: {}", e);
            Vec::new()
        }
    }
}

/// 查询端口代理列表但不打印 netsh 输出（用于文件备份等，避免刷屏或干扰日志）。
pub fn get_all_ipv4_to_ipv4_port_proxy_silent() -> Vec<PortForwarding> {
    match run_commands(PORT_PROXY_V4TOV4, RunCommandOptions::new(true, false, false)) {
        Ok(out) => from_cmd_load_port_forwarding(&out.to_string()),
        Err(e) => {
            println!("get_all_ipv4_to_ipv4_port_proxy_silent failed: {}", e);
            Vec::new()
        }
    }
}

pub fn from_cmd_load_port_forwarding(cmd_string: &String) -> Vec<PortForwarding> {
    let mut res: Vec<PortForwarding> = Vec::new();
    for line in cmd_string.lines() {
        let line = line.trim_end();
        let data: Vec<String> = line.split_whitespace().map(|x| x.to_string()).collect();
        if data.len() == 4 {
            let listen_address = data[0].clone();
            let listen_port = data[1].parse::<i64>();
            let connect_address = data[2].clone();
            let connect_port = data[3].parse::<i64>();

            let address_is_ok =
                check_ipv4_by_string(&listen_address) && check_ipv4_by_string(&connect_address);
            let port_is_ok = listen_port.is_ok() && connect_port.is_ok();

            if address_is_ok && port_is_ok {
                let listen_port = listen_port.unwrap();
                let connect_port = connect_port.unwrap();
                if !(1..=65535).contains(&listen_port) || !(1..=65535).contains(&connect_port) {
                    println!("Invalid port range {}", Red.paint(line));
                    continue;
                }
                let port = PortForwarding::new(
                    (listen_address, listen_port),
                    (connect_address, connect_port),
                );
                res.push(port);
            } else if check_ipv4_by_string(&listen_address) {
                // 表头等也会被拆成 4 段，但首列不是 IPv4；仅对疑似数据行告警。
                println!("Invalid port found {}", Red.paint(line));
            }
        }
    }
    res
}

/// 从输入的cmd字符串中获取信息
pub fn from_cmd_data(data: Vec<String>) -> Result<PortForwarding, String> {
    if data.len() != 4 {
        return Err("需传入4个数据".to_string());
    }
    let listen_port = data[1]
        .parse::<i64>()
        .map_err(|e| format!("listen port: {}", e))?;
    let connect_port = data[3]
        .parse::<i64>()
        .map_err(|e| format!("connect port: {}", e))?;
    Ok(PortForwarding::new(
        (data[0].clone(), listen_port),
        (data[2].clone(), connect_port),
    ))
}
