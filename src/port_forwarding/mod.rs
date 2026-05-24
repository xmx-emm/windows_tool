//! Windows `netsh interface portproxy` 的封装（当前仅 IPv4→IPv4）。
//!
//! - **查询**当前规则列表通常**不需要**管理员权限。
//! - **添加、删除、重置**等写操作一般需要**管理员权限**。
//!
//! 子命令概览：`add`、`delete`、`dump`、`reset`、`set`、`show`（详见 `netsh interface portproxy /?`）。
//!
//! # 示例
//!
//! ```no_run
//! use windows_tool::port_forwarding::PortForwarding;
//!
//! PortForwarding::reset();
//! let item = PortForwarding::new(("127.0.0.1", 100), ("127.0.0.1", 100));
//! item.forward();
//! let _ = item.check();
//! let items = PortForwarding::get_ipv4_to_ipv4();
//! println!("{:?}", items);
//! ```

use crate::port_forwarding::cmd::add_cmd;
use crate::port_forwarding::ipv::Ipv;
use crate::utils::{check_ipv4_by_string, run_multiple_commands, RunCommandOptions};
use serde::{Deserialize, Serialize};
use std::cmp::{Eq, PartialEq};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

pub mod cmd;
pub mod command;
pub mod ipv;
pub mod backups;
/// #[derive(Serialize, Deserialize, Debug)]
/// pub enum IPVType {
///     IPV4(u8, u8, u8, u8), //127.0.0.1
///     IPV6(String),         //2001:0db8:85a3:0000:0000:8a2e:0370:7334
/// }

/// 单条 IPv4 端口转发规则（监听地址/端口 → 连接地址/端口）。
///
/// 当前仅支持 **v4tov4**；IPv6 见模块级说明中的 TODO。
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
        run_multiple_commands(&vs, RunCommandOptions::new(false, true, true));
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
