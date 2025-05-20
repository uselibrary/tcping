# TCP Ping

TCP Ping 是一个简单而强大的工具，用于测试 TCP 端口的连通性。与传统的 ICMP ping 不同，TCP Ping 测试特定端口的 TCP 连接，这对于检测服务可用性和网络连通性非常有用。

## 功能特点

- 测试任何 TCP 端口的连通性
- 支持域名和 IP 地址（IPv4 和 IPv6）
- 可指定连接超时时间
- 可指定 ping 测试次数
- 可指定测试间隔时间
- 提供详细的统计信息（成功率、往返时间等）
- 支持彩色输出，直观显示连接状态
- 详细模式提供更多诊断信息

## 安装方法

从 [GitHub Releases](https://github.com/uselibrary/tcping/releases) 页面下载最新版本的二进制文件，重命名为`tcping`，放到系统默认位置即可使用，如：
- Linux：/usr/local/bin
- macOS：/usr/local/bin
- Windows：C:\Windows\System32 或特定位置并添加到 PATH 环境变量中

Linux版本有多个后缀，其中：`-static`表示静态编译，表示使用 musl libc 编译，它是一个静态编译的二进制文件，不依赖于系统库，可以在任何 Linux 系统上运行，尤其是alpine linux等精简版系统上。

## 使用方法

```
tcping [选项] <主机名/IP地址>
```

### 命令行选项

```
选项:
  -p, --port <PORT>          目标端口号 [默认值: 80]
  -n, --count <COUNT>        Ping的次数 (0表示无限次) [默认值: 0]
  -t, --timeout <TIMEOUT>    每次Ping的超时时间(毫秒) [默认值: 1000]
  -i, --interval <INTERVAL>  两次Ping之间的间隔时间(毫秒) [默认值: 1000]
  -4                         强制使用IPv4
  -6                         强制使用IPv6
  -v, --verbose              启用详细输出模式
  -c, --color                启用彩色输出模式
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
tcping -p 443 www.example.com
```

### 指定测试次数

```bash
tcping -n 5 www.example.com
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

### 启用详细输出模式

```bash
tcping -v www.example.com
```

### 启用彩色输出

```bash
tcping -c www.example.com
```

## 输出示例

### 标准输出

```
正在对 www.example.com (IPv4 - 93.184.216.34) 端口 80 执行 TCP Ping
从 www.example.com:80 收到响应: seq=0 time=125.37ms
从 www.example.com:80 收到响应: seq=1 time=124.19ms
从 www.example.com:80 收到响应: seq=2 time=124.02ms
从 www.example.com:80 超时: seq=3
从 www.example.com:80 收到响应: seq=4 time=125.12ms

--- www.example.com TCP ping 统计 ---
已发送 = 5, 已接收 = 4, 丢失 = 1 (20.0% 丢失)
往返时间(RTT): 最小 = 124.02ms, 最大 = 125.37ms, 平均 = 124.68ms
```

### 详细模式输出

```
开始解析主机: www.example.com
成功解析主机, 找到 2 个IP地址
应用IP版本过滤: 仅IPv4
正在对 www.example.com (IPv4 - 93.184.216.34) 端口 80 执行 TCP Ping
测试参数: 超时=1000 ms, 间隔=1000 ms, 测试次数=5
从 www.example.com:80 收到响应: seq=0 time=125.37ms
  -> 本地连接详情: 192.168.1.2:54321 -> 93.184.216.34:80
从 www.example.com:80 收到响应: seq=1 time=124.19ms
  -> 本地连接详情: 192.168.1.2:54322 -> 93.184.216.34:80
从 www.example.com:80 收到响应: seq=2 time=124.02ms
  -> 本地连接详情: 192.168.1.2:54323 -> 93.184.216.34:80
从 www.example.com:80 超时: seq=3
  -> 连接失败详情: 连接超时
从 www.example.com:80 收到响应: seq=4 time=125.12ms
  -> 本地连接详情: 192.168.1.2:54324 -> 93.184.216.34:80

--- www.example.com TCP ping 统计 ---
已发送 = 5, 已接收 = 4, 丢失 = 1 (20.0% 丢失)
往返时间(RTT): 最小 = 124.02ms, 最大 = 125.37ms, 平均 = 124.68ms
中位数(Median) = 124.66ms
标准差(StdDev) = 0.63ms
抖动(Jitter) = 0.75ms
```

## 程序退出

使用 `Ctrl+C` 可以随时终止程序，程序会显示已收集的统计信息。连续按两次 `Ctrl+C` 将立即退出程序（不显示统计信息）。

## CI
只有当tag包含`v`时，CI才会运行。例如`v0.1.6`。

在本地确认 Tag 存在
```bash
git tag | grep v0.1.6
```

如果没有输出，说明你本地还没建这个 tag，需要先：
```bash
git tag -a v0.1.6 -m "Release v0.1.6"
```

推送这个 Tag 到远程

```bash
git push origin v0.1.6
```

或者一次性推所有本地新 tag：
```bash
git push --tags
```

验证 Tag 已推成功
```bash
git ls-remote --tags origin | grep v0.1.6
```

如果能看到类似 refs/tags/v0.1.6 的一行，就说明已经推上去了。

## 编译方法

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

生成的二进制文件位于`target/x86_64-unknown-linux-musl/release/tcping`，它不依赖任何系统库，可以在任何 Linux 系统上运行。这种方法最简单且效果最好，特别是对于需要在各种环境中分发的命令行工具。

## 许可证
本项目使用GPL-3.0许可证。有关详细信息，请参阅 LICENSE 文件。

