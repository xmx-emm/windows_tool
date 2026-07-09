use ansi_term::Color::{Green, Red};
use encoding_rs::GBK;
use std::process::Output;

pub trait Println {
    fn print_ln(&self);
    fn to_string(&self) -> String;
}

/// 经 PowerShell 拉起的子进程 stdout 常为 UTF-8；直接跑在 cmd 下多为系统 ANSI（中文 Windows 常为 GBK）。
pub fn decode_process_output(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => GBK.decode(bytes).0.into_owned(),
    }
}

impl Println for Output {
    /// 按 UTF-8 优先、否则 GBK 解码后打印，避免控制台中文乱码。
    fn print_ln(&self) {
        let text = decode_process_output(&self.stdout);
        text.split("\r\n").for_each(|x| {
            println!("{}", Green.paint(x));
        });
        println!(
            "Exit code {}\n",
            Red.paint(self.status.code().unwrap_or(-1).to_string())
        );
    }

    fn to_string(&self) -> String {
        decode_process_output(&self.stdout)
    }
}
