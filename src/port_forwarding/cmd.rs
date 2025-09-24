use crate::port_forwarding::PortForwarding;

/// 参数:
/// 标记            值
/// listenaddress   - IPv4 侦听地址。
/// listenport      - IPv4 侦听端口。
/// connectaddress  - IPv4 连接地址。
/// connectport     - IPv4 连接端口。
/// protocol        - 使用的协议。现在只支持 TCP。
/// 说明: 添加通过 IPv4 的 IPv4 和代理连接到的侦听项目。
/// Require administrator privileges to execute
pub fn add_cmd(forward: &PortForwarding) -> String {
    let listen_address = &forward.listen.address;
    let listen_port = forward.listen.port;
    let connect_address = &forward.connect.address;
    let connect_port = forward.connect.port;
    format!(
        "netsh interface portproxy add v4tov4 listenaddress={} listenport={}  connectaddress={} connectport={}",
        listen_address, listen_port, connect_address, connect_port,
    )
}

///
/// netsh interface portproxy delete v4tov4 listenaddress=127.0.0.1 listenport=100 protocol=tcp
pub fn del_cmd(address: &String, port: &i64) -> String {
    format!(
        "netsh interface portproxy del v4tov4 listenaddress={} listenport={}",
        &address, port,
    )
}

///netsh interface portproxy set v4tov4
/// 用法: set v4tov4 [listenport=]<integer>|<servicename>
/// [connectaddress=]<IPv4 address>|<hostname>
/// [[connectport=]<integer>|<servicename>]
/// [[listenaddress=]<IPv4 address>|<hostname>]
/// [[protocol=]tcp]
/// 参数:
/// 标记            值
/// listenport      - IPv4 侦听端口。
/// connectaddress  - IPv4 连接地址。
/// connectport     - IPv4 连接端口。
/// listenaddress   - IPv4 侦听地址。
/// protocol        - 使用的协议。现在只支持 TCP。
/// 说明: 更新通过 IPv4 的 IPv4 和代理连接到的侦听项目。
pub fn set_cmd(port: &PortForwarding) -> String {
    let listen_address = &port.listen.address;
    let listen_port = port.listen.port;
    let connect_address = &port.connect.address;
    let connect_port = port.connect.port;
    format!(
        "netsh interface portproxy set v4tov4 listenaddress={} listenport={}  connectaddress={} connectport={}",
        listen_address, listen_port, connect_address, connect_port
    )
}

pub static RESET_CMD: &str = "netsh interface portproxy reset";

pub static PORT_PROXY_V4TOV4: &str = "netsh interface portproxy show v4tov4";
