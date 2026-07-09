use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ipv {
    // ipv_address: IPVType, //Ipv类型
    pub address: String, //地址
    pub port: i64,       //端口
}

impl Ipv {
    pub fn new<S: AsRef<str>>(address: S, port: i64) -> Ipv {
        Ipv { address: address.as_ref().to_string(), port }
    }
}

impl Display for Ipv {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "地址:{a},端口:{p}", a = self.address, p = self.port)
    }
}
