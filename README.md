# TCP Ping

TCP Ping 是一个简单而强大的工具，用于测试 TCP 端口的连通性。与传统的 ICMP ping 不同，TCP Ping 测试特定端口的 TCP 连接，这对于检测服务可用性和网络连通性非常有用。

## 功能特点

- 测试任何 TCP 端口的连通性
- 支持域名和 IP 地址（IPv4 和 IPv6）
- 可指定连接超时时间
- 可指定 ping 测试次数
- 可指定测试间隔时间
- 提供详细的统计信息（成功率、往返时间等）

## 安装方法

### 从源码编译

需要先安装 Rust 开发环境。如果尚未安装，请访问 [rustup.rs](https://rustup.rs) 按照指引安装。

```bash
# 克隆仓库
git clone https://github.com/uselibrary/tcping.git
cd tcping

# 编译
cargo build --release

# 安装到系统路径（可选）
cargo install --path .
```

编译好的可执行文件位于 `target/release/tcping`。


### 静态编译（不依赖系统库）

如果你想要编译一个不依赖系统库的静态二进制文件，可以使用 MUSL 目标：

安装 MUSL 目标
```bash
rustup target add x86_64-unknown-linux-musl
```

静态编译
```bash
cargo build --release --target=x86_64-unknown-linux-musl
```

生成的二进制文件位于`target/x86_64-unknown-linux-musl/release/tcping`，它不依赖任何系统库，可以在任何 Linux 系统上运行。这种方法最简单且效果最好，特别是对于需要在各种环境中分发的命令行工具。这种方法最简单且效果最好，特别是对于需要在各种环境中分发的命令行工具。

## 使用方法

```
tcping [选项] <主机名/IP地址>
```

### 命令行选项

```
选项:
  -p, --port <PORT>          目标端口号 [默认值: 80]
  -c, --count <COUNT>        Ping的次数 (0表示无限次) [默认值: 0]
  -t, --timeout <TIMEOUT>    每次Ping的超时时间(毫秒) [默认值: 1000]
  -i, --interval <INTERVAL>  两次Ping之间的间隔时间(毫秒) [默认值: 1000]
  -4                         强制使用IPv4
  -6                         强制使用IPv6
  -h, --help                 显示帮助信息
  -V, --version              显示版本信息
```

## 使用示例

### 测试网站的 HTTP 端口

```bash
tcping www.example.com
```

### 测试特定端口

```bash
tcping --port 443 www.example.com
```

### 指定测试次数

```bash
tcping -c 5 www.example.com
```

### 指定超时和间隔时间

```bash
tcping -t 2000 -i 500 www.example.com
```

### 强制使用 IPv4 或 IPv6

```bash
tcping -4 www.example.com  # 仅使用 IPv4
tcping -6 www.example.com  # 仅使用 IPv6
```

## 输出示例

```
正在对 www.example.com (IPv4 - 93.184.216.34) 端口 80 执行 TCP Ping
从 www.example.com:80 收到响应: seq=0 time=125.37ms
从 www.example.com:80 收到响应: seq=1 time=124.19ms
从 www.example.com:80 收到响应: seq=2 time=124.02ms
从 www.example.com:80 超时: seq=3
从 www.example.com:80 收到响应: seq=4 time=125.12ms

--- 目标主机 TCP ping 统计 ---
已发送 = 5, 已接收 = 4, 丢失 = 1 (20.0% 丢失)
往返时间(RTT): 最小 = 124.02ms, 最大 = 125.37ms, 平均 = 124.68ms
```

## 程序退出

使用 `Ctrl+C` 可以随时终止程序，即使它被配置为运行无限次 ping。

## 许可证
本项目使用GPL-3.0许可证。有关详细信息，请参阅 LICENSE 文件。
```
