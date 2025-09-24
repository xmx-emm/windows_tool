这是一个Window工具
```

steam 输入的id及游戏的id都使用usize进行输入
```

端口转发 port_forwarding
```rust
// 可用的代码
// powershell -Command \"Start-Process cmd '/k netsh interface portproxy add v4tov4 listenport=4000 listenaddress=10.0.0.113 connectaddress=192.168.21.4 connectport=22 && netsh interface portproxy add v4tov4 listenport=666 listenaddress=10.0.0.113 connectaddress=192.168.21.4 connectport=22'\" -Verb RunAs
```
```
多条命令行运行在rust
let output = Command::new("powershell")
    .arg("powershell -Command \"Start-Process cmd '/k netsh interface portproxy add v4tov4 listenport=400 listenaddress=10.0.0.113 connectaddress=192.168.21.4 connectport=22 && netsh interface portproxy add v4tov4 listenport=666 listenaddress=10.0.0.113 connectaddress=192.168.21.4 connectport=22'\" -Verb RunAs")
    .output()
    .expect("add error");
output.print_ln();
println!("Add added 1");
```: