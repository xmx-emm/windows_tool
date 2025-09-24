use crate::port_forwarding::cmd::add_cmd;
use crate::port_forwarding::ipv::Ipv;
use crate::utils::{check_ipv4_by_string, run_multiple_commands};
use serde::{Deserialize, Serialize};
use std::cmp::{Eq, PartialEq};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// netsh interface portproxy show all
/// netsh interface portproxy add v4tov4 listenaddress=127.0.0.1 listenport=100  connectaddress=127.1.1.0 connectport=120
/// netsh interface portproxy delete v4tov4 listenaddress=127.0.0.1 listenport=100 protocol=tcp
/// 端口代理/端口转发
/// 通过cmd来获取window的端口转发设置
/// 添加/删除/修改/重置 操作都需要管理员权限
/// Tips: 只有获取不需要管理员权限

/// netsh interface portproxy
/// add            - 在一个表格中添加一个配置项。
/// delete         - 从一个表格中删除一个配置项。 del
/// dump           - 显示一个配置脚本。
/// help           - 显示命令列表。
/// reset          - 重置端口代理配置状态。
/// set            - 设置配置信息。
/// show           - 显示信息。
/// ```
/// use windows_tool::port_forwarding::ipv::Ipv;
/// use windows_tool::port_forwarding::PortForwarding;
/// PortForwarding::reset();
/// let item = PortForwarding::new(("127.0.0.1".to_string(),100),("127.0.0.1".to_string(),100));
/// &item.forward();
/// &item.check();
/// let items = PortForwarding::get_ipv4_to_ipv4();
/// println!("aa {:?}", items)
/// ```
///
pub mod cmd;
pub mod command;
pub mod ipv;
pub mod backups;
// #[derive(Serialize, Deserialize, Debug)]
// pub enum IPVType {
//     IPV4(u8, u8, u8, u8), //127.0.0.1
//     IPV6(String),         //2001:0db8:85a3:0000:0000:8a2e:0370:7334
// }

/// window 端口转发
/// TODO 目前只支持ipv4端口转发
#[derive(Serialize, Deserialize, Debug)]
pub struct PortForwarding {
    pub listen: Ipv,  //from 从
    pub connect: Ipv, //to 到
}

impl PortForwarding {
    /// 获取所有的ipv4 to ipv4的端口转发
    pub fn get_ipv4_to_ipv4() -> Vec<PortForwarding> {
        command::get_all_ipv4_to_ipv4_port_proxy()
    }
    /// 删除所有的端口转发
    pub fn reset() {
        command::reset()
    }
    ///设置单个端口转发
    pub fn set(po: &PortForwarding) {
        command::set(po)
    }

    /// 添加端口转发
    pub fn new<S: AsRef<str>>(listen_from: (S, i64), connect_to: (S, i64)) -> Self {
        let listen = Ipv::new(listen_from.0, listen_from.1);
        let connect = Ipv::new(connect_to.0, connect_to.1);
        PortForwarding { listen, connect }
    }
    /// 添加多条端口转发
    pub fn new_multiple(list: Vec<PortForwarding>) {
        let vs = list
            .into_iter()
            .filter(PortForwarding::check_ipv_address) //检查地址是否正确
            .collect::<Vec<PortForwarding>>()
            .into_iter()
            .map(|x| add_cmd(&x)) //转换为cmd
            .collect::<Vec<String>>();
        run_multiple_commands(&vs, false, true);
    }

    /// 转发端口
    pub fn forward(&self) {
        command::add(self);
    }

    /// 删除端口转发
    pub fn del(&self) {
        let listen = &self.listen;
        command::del(&listen.address, &listen.port);
    }

    /// 检查当前转发是否已经在转发列表内
    pub fn check(&self) -> bool {
        // 检查端口是否可用
        let items = command::get_all_ipv4_to_ipv4_port_proxy();
        for item in items {
            if item == *self {
                return true;
            }
        }
        false
    }

    /// 检查IPV地址是否正确
    pub fn check_ipv_address(&self) -> bool {
        check_ipv4_by_string(&self.listen.address) && check_ipv4_by_string(&self.connect.address)
    }
}

impl Display for PortForwarding {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let listen = &self.listen;
        let connect = &self.connect;
        write!(
            f,
            "PortForwarding(from {}:{},to {}:{})",
            listen.address, listen.port, connect.address, connect.port
        )
    }
}

//实现 ==
impl PartialEq for PortForwarding {
    fn eq(&self, other: &Self) -> bool {
        self.listen.address == other.listen.address
            && self.listen.port == other.listen.port
            && self.connect.address == other.connect.address
            && self.connect.port == other.connect.port
    }
}

impl Eq for PortForwarding {}
impl Hash for PortForwarding {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.listen.address.hash(state);
        self.listen.port.hash(state);
        self.connect.address.hash(state);
        self.connect.port.hash(state);
    }
}
