use crate::port_forwarding::PortForwarding;
use crate::port_forwarding::cmd::{PORT_PROXY_V4TOV4, RESET_CMD, add_cmd, del_cmd, set_cmd};
use crate::utils::{Println, check_ipv4_by_string, run_commands};
use ansi_term::Color::Red;

///
/// netsh interface portproxy add v4tov4 listenport=40005 listenaddress=10.0.0.113 connectaddress=192.168.21.4 connectport=22
///
pub fn add(forward: &PortForwarding) {
    let cmd = add_cmd(forward);
    let out = run_commands(&cmd, true, true);
    out.print_ln();
}

/// 删除一条项目(需要管理员权限)
/// netsh interface portproxy del v4tov4 listenport listenaddress
/// 此上下文中的命令:
/// delete v4tov4  - 删除通过 IPv4 的 IPv4 和代理连接到的侦听项目。
/// delete v4tov6  - 删除通过 IPv6 的 IPv4 和代理连接到的侦听项目。
/// delete v6tov4  - 删除通过 IPv4 的 IPv6 和代理连接到的侦听项目。
/// delete v6tov6  - 删除通过 IPv6 的 IPv6 和代理连接到的侦听项目。
/// Require administrator privileges to execute
pub fn del(address: &String, port: &i64) {
    let cmd = del_cmd(address, port);
    let out = run_commands(&cmd, true, true);
    out.print_ln();
}

/// 命令行
/// 重置(需要管理员权限)
/// Require administrator privileges to execute
pub fn reset() {
    let cmd = RESET_CMD;
    let out = run_commands(&cmd.to_string(), true, true);
    out.print_ln();
}

///设置代理
pub fn set(port: &PortForwarding) {
    let cmd = set_cmd(port);
    let out = run_commands(&cmd, true, true);
    out.print_ln();
}

/// 获取所有的ipv4 to ipv4代理
/// ```
/// use windows_tool::port_forwarding::command::get_all_ipv4_to_ipv4_port_proxy;
/// let pp = get_all_ipv4_to_ipv4_port_proxy();
/// println!("pp {:?}",pp);
/// ```
pub fn get_all_ipv4_to_ipv4_port_proxy() -> Vec<PortForwarding> {
    let out = run_commands(PORT_PROXY_V4TOV4, true, false);
    out.print_ln();
    from_cmd_load_port_forwarding(&out.to_string())
}

/// 侦听 ipv4:                 连接到 ipv4:
///
/// 地址            端口        地址            端口
/// --------------- ----------  --------------- ----------
/// 10.0.0.113      40005       192.168.21.4    22
/// 10.0.0.113      4000        192.168.21.4    22
/// 10.0.0.113      400         192.168.21.4    22
pub fn from_cmd_load_port_forwarding(cmd_string: &String) -> Vec<PortForwarding> {
    let mut res: Vec<PortForwarding> = Vec::new();
    for line in cmd_string.split("\r\n") {
        let data: Vec<String> = line.split_whitespace().map(|x| x.to_string()).collect();
        if data.len() == 4 {
            let listen_address = data.get(0).unwrap().to_string();
            let listen_port = data.get(1).unwrap().parse::<i64>();
            let connect_address = data.get(2).unwrap().to_string();
            let connect_port = data.get(3).unwrap().parse::<i64>();

            let address_is_ok =
                check_ipv4_by_string(&listen_address) && check_ipv4_by_string(&connect_address);
            let port_is_ok = listen_port.is_ok() && connect_port.is_ok();

            if address_is_ok && port_is_ok {
                let port = PortForwarding::new(
                    (listen_address, listen_port.unwrap()),
                    (connect_address, connect_port.unwrap()),
                );
                res.push(port);
            } else {
                println!("Invalid port found {}", Red.paint(line));
            }
        }
    }
    res
}

/// 从输入的cmd字符串中获取信息
/// 通过解析字符串并拆分来获取所有ipv4的端口转发
pub fn from_cmd_data(data: Vec<String>) -> PortForwarding {
    assert_eq!(data.len(), 4, "需传入4个数据! ");
    let from = (data[0].to_string(), data[1].parse::<i64>().unwrap());
    let to = (data[2].to_string(), data[3].parse::<i64>().unwrap());
    PortForwarding::new(from, to)
}
