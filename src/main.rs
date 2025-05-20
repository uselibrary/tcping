use clap::Parser;
use colored::Colorize;
use dns_lookup::lookup_host;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;


/// TCP Ping - 测试TCP端口连通性的工具
#[derive(Parser, Debug)]
#[clap(author, version, about, after_help = "\
使用示例:
  tcping www.example.com                   测试网站的HTTP端口(默认80)
  tcping www.example.com -p 443            测试指定端口(HTTPS)
  tcping -n 5 www.example.com              指定测试次数(5次)
  tcping -t 2000 -i 500 www.example.com    设置超时2秒和间隔0.5秒
  tcping -4 www.example.com                强制使用IPv4
  tcping -6 www.example.com                强制使用IPv6
  tcping -v www.example.com                启用详细输出模式
  tcping -c www.example.com                启用彩色输出模式")]
struct Args {
    /// 目标主机名或IP地址
    #[clap(required = true)]
    host: String,

    /// 目标端口号
    #[clap(short, long, default_value = "80")]
    port: u16,

    /// Ping的次数 (0表示无限次)
    #[clap(short = 'n', long = "count", default_value = "0")]
    count: u32,

    /// 每次Ping的超时时间(毫秒)
    #[clap(short, long, default_value = "1000")]
    timeout: u64,

    /// 两次Ping之间的间隔时间(毫秒)
    #[clap(short = 'i', long, default_value = "1000")]
    interval: u64,

    /// 强制使用IPv4
    #[clap(short = '4', long, conflicts_with = "ipv6")]
    ipv4: bool,

    /// 强制使用IPv6
    #[clap(short = '6', long, conflicts_with = "ipv4")]
    ipv6: bool,
    
    /// 启用详细输出模式
    #[clap(short, long)]
    verbose: bool,
    
    /// 启用彩色输出模式
    #[clap(short = 'c', long = "color")]
    color: bool,
}

struct PingStats {
    transmitted: u32,
    received: u32,
    total_time: Duration,
    min_time: Option<Duration>,
    max_time: Duration,
    // 新增：存储所有RTT值用于计算统计数据
    rtt_values: Vec<Duration>,
}

impl PingStats {
    fn new() -> Self {
        PingStats {
            transmitted: 0,
            received: 0,
            total_time: Duration::from_secs(0),
            min_time: None,
            max_time: Duration::from_secs(0),
            rtt_values: Vec::new(), // 初始化RTT值列表
        }
    }

    fn update(&mut self, success: bool, rtt: Option<Duration>) {
        self.transmitted += 1;
        
        if success {
            self.received += 1;
            
            if let Some(time) = rtt {
                self.total_time += time;
                self.rtt_values.push(time); // 保存每次RTT值
                
                // 更新最小时间，处理 None 情况
                self.min_time = match self.min_time {
                    None => Some(time),
                    Some(current_min) if time < current_min => Some(time),
                    other => other,
                };
                
                if time > self.max_time {
                    self.max_time = time;
                }
            }
        }
    }

    // 计算中位数
    fn median_time(&self) -> Option<Duration> {
        if self.rtt_values.is_empty() {
            return None;
        }
        
        let mut sorted_values = self.rtt_values.clone();
        sorted_values.sort();
        
        let mid = sorted_values.len() / 2;
        if sorted_values.len() % 2 == 0 && sorted_values.len() >= 2 {
            // 偶数个元素，取中间两个的平均值
            let sum = sorted_values[mid - 1] + sorted_values[mid];
            Some(sum / 2)
        } else {
            // 奇数个元素，直接取中间值
            Some(sorted_values[mid])
        }
    }

    // 计算标准差
    fn std_deviation(&self) -> Option<f64> {
        if self.rtt_values.len() <= 1 {
            return None; // 至少需要两个样本计算标准差
        }
        
        let mean = self.total_time.as_secs_f64() / self.received as f64;
        
        let variance = self.rtt_values.iter()
            .map(|&time| {
                let diff = time.as_secs_f64() - mean;
                diff * diff
            })
            .sum::<f64>() / (self.rtt_values.len() - 1) as f64; // 样本标准差使用n-1
        
        Some(variance.sqrt())
    }

    fn print_summary(&self, hostname: &str, verbose: bool) {
        println!("\n--- {} TCP ping 统计 ---", hostname);
        println!("已发送 = {}, 已接收 = {}, 丢失 = {} ({:.1}% 丢失)",
            self.transmitted,
            self.received,
            self.transmitted - self.received,
            if self.transmitted > 0 {
                (self.transmitted as f64 - self.received as f64) / self.transmitted as f64 * 100.0
            } else {
                0.0
            });
        
        if self.received > 0 {
            let avg_time = self.total_time / self.received;
            // 处理 min_time 为 None 的情况
            let min_time = self.min_time.unwrap_or(Duration::from_secs(0));
            
            println!("往返时间(RTT): 最小 = {:.2}ms, 最大 = {:.2}ms, 平均 = {:.2}ms",
                min_time.as_secs_f64() * 1000.0,
                self.max_time.as_secs_f64() * 1000.0,
                avg_time.as_secs_f64() * 1000.0);
            
            // 在详细模式下显示额外统计信息
            if verbose && self.received >= 2 {
                if let Some(median) = self.median_time() {
                    println!("中位数(Median) = {:.2}ms", median.as_secs_f64() * 1000.0);
                }
                
                if let Some(std_dev) = self.std_deviation() {
                    println!("标准差(StdDev) = {:.2}ms", std_dev * 1000.0);
                }
            }
        }
    }
}

// 添加一个辅助函数来格式化主机地址和端口
fn format_host_port(host: &str, port: u16) -> String {
    // 检查主机名是否是IPv6地址
    if let Ok(addr) = host.parse::<IpAddr>() {
        if addr.is_ipv6() {
            return format!("[{}]:{}", host, port);
        }
    }
    format!("{}:{}", host, port)
}

/// 创建TCP连接并返回连接结果和本地地址信息
async fn tcp_connect(
    addr: &SocketAddr,
    timeout_ms: u64
) -> Result<Option<SocketAddr>, String> {
    match tokio::time::timeout(
        Duration::from_millis(timeout_ms), 
        TcpStream::connect(*addr)
    ).await {
        Ok(Ok(stream)) => {
            match stream.local_addr() {
                Ok(local_addr) => Ok(Some(local_addr)),
                Err(_) => Ok(None),
            }
        },
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("连接超时".into()),
    }
}

/// 统一处理错误信息的打印
fn print_error(message: &str, color: bool) {
    if color {
        eprintln!("{}", message.red());
    } else {
        eprintln!("{}", message);
    }
}

// 提取IP地址筛选逻辑为单独函数
fn filter_ip_addresses(ip_addrs: Vec<IpAddr>, ipv4: bool, ipv6: bool, verbose: bool) -> Vec<IpAddr> {
    if verbose {
        println!("应用IP版本过滤: {}", 
            if ipv4 { "仅IPv4" } else if ipv6 { "仅IPv6" } else { "自动选择" });
    }

    // 优化：直接使用迭代器筛选，避免创建中间向量
    if ipv4 {
        ip_addrs.into_iter().filter(|ip| ip.is_ipv4()).collect()
    } else if ipv6 {
        ip_addrs.into_iter().filter(|ip| ip.is_ipv6()).collect()
    } else {
        // 优先返回IPv4地址，若无IPv4则返回IPv6
        let v4 = ip_addrs.iter().find(|ip| ip.is_ipv4());
        if v4.is_some() {
            if verbose && ip_addrs.iter().any(|ip| ip.is_ipv6()) {
                println!("找到IPv4和IPv6地址, 优先使用IPv4");
            }
            ip_addrs.into_iter().filter(|ip| ip.is_ipv4()).collect()
        } else {
            ip_addrs.into_iter().filter(|ip| ip.is_ipv6()).collect()
        }
    }
}

// 新增：解析主机名并返回IP地址列表
fn resolve_host(host: &str, ipv4: bool, ipv6: bool, verbose: bool) -> Result<Vec<IpAddr>, String> {
    if verbose {
        println!("开始解析主机: {}", host);
    }

    let ip_addrs = match lookup_host(host) {
        Ok(ips) => {
            if verbose {
                println!("成功解析主机, 找到 {} 个IP地址", ips.len());
            }
            ips
        },
        Err(e) => {
            if verbose {
                println!("DNS解析失败: {}, 尝试作为原始IP处理", e);
            }
            match host.parse::<IpAddr>() {
                Ok(ip) => {
                    if verbose {
                        println!("输入被识别为有效的IP地址");
                    }
                    vec![ip]
                },
                Err(_) => {
                    return Err(format!("无法解析主机名: {}", host));
                }
            }
        }
    };

    let filtered_ips = filter_ip_addresses(ip_addrs, ipv4, ipv6, verbose);

    if filtered_ips.is_empty() {
        let version = if ipv6 { "IPv6" } else { "IPv4" };
        return Err(format!("未找到主机 {} 的{}地址", host, version));
    }

    if verbose && filtered_ips.len() > 1 {
        println!("找到多个IP地址, 使用第一个: {}", filtered_ips[0]);
        println!("其他备选IP: {:?}", &filtered_ips[1..]);
    }

    Ok(filtered_ips)
}

// 改进信号处理逻辑，更加简洁
fn setup_signal_handler(running: Arc<AtomicBool>) {
    ctrlc::set_handler(move || {
        if running.swap(false, Ordering::SeqCst) {
            println!(); // 只在第一次按Ctrl+C时打印新行
        } else {
            std::process::exit(0); // 连续两次按Ctrl+C直接退出
        }
    }).expect("无法设置信号处理程序");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // 使用新的主机解析函数
    let filtered_ips = match resolve_host(&args.host, args.ipv4, args.ipv6, args.verbose) {
        Ok(ips) => ips,
        Err(e) => {
            print_error(&e, args.color);
            return Ok(());
        }
    };

    let ip = filtered_ips[0]; // 已确保至少有一个IP
    let port = args.port;

    let hostname = args.host.clone();
    let running = Arc::new(AtomicBool::new(true));
    setup_signal_handler(running.clone());

    let mut stats = PingStats::new();
    let target = SocketAddr::new(ip, port);

    println!("正在对 {} ({} - {}) 端口 {} 执行 TCP Ping", 
        args.host, if ip.is_ipv4() { "IPv4" } else { "IPv6" }, ip, port);

    if args.verbose {
        println!("测试参数: 超时={} ms, 间隔={} ms, 测试次数={}", 
            args.timeout, args.interval, if args.count == 0 { "无限".to_string() } else { args.count.to_string() });
    }

    let mut seq = 0;

    while running.load(Ordering::SeqCst) {
        if args.count > 0 && seq >= args.count {
            break;
        }

        let start = Instant::now();
        if !running.load(Ordering::SeqCst) {
            break;
        }

        let connect_task = tokio::spawn(async move {
            tcp_connect(&target, args.timeout).await
        });

        let result = if running.load(Ordering::SeqCst) {
            match tokio::time::timeout(Duration::from_millis(args.timeout), connect_task).await {
                Ok(Ok(res)) => res,
                Ok(Err(_)) => Err("任务被取消".into()),
                Err(_) => Err("连接超时".into()),
            }
        } else {
            break;
        };

        if !running.load(Ordering::SeqCst) {
            break;
        }

        let elapsed = start.elapsed();
        let seq_num = seq;
        seq += 1;

        match result {
            Ok(local_addr) => {
                let success_msg = format!("从 {} 收到响应: seq={} time={:.2}ms",
                    format_host_port(&hostname, port), seq_num, elapsed.as_secs_f64() * 1000.0);
                println!("{}", if args.color { success_msg.green().to_string() } else { success_msg });

                if args.verbose {
                    if let Some(addr) = local_addr {
                        println!("  -> 本地连接详情: {} -> {}", addr, target);
                    } else {
                        println!("  -> 无法获取本地连接信息");
                    }
                }

                stats.update(true, Some(elapsed));
            },
            Err(err) => {
                let error_msg = if err.contains("timed out") || err.contains("超时") {
                    format!("从 {} 超时: seq={}", format_host_port(&hostname, port), seq_num)
                } else {
                    format!("从 {} 无法连接: seq={}", format_host_port(&hostname, port), seq_num)
                };
                println!("{}", if args.color { error_msg.red().to_string() } else { error_msg });

                if args.verbose {
                    println!("  -> 连接失败详情: {}", err);
                }
                stats.update(false, None);
            }
        }

        if !running.load(Ordering::SeqCst) {
            break;
        }

        if seq < args.count || args.count == 0 {
            tokio::time::sleep(Duration::from_millis(args.interval)).await;
        }
    }

    if stats.transmitted > 0 {
        stats.print_summary(&hostname, args.verbose);
    }

    Ok(())
}