use clap::Parser;
use colored::Colorize;
use std::net::SocketAddr;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

mod cli;
mod network;
mod stats;
mod utils;

use cli::Args;
use network::{resolve_host, tcp_connect};
use stats::PingStats;
use utils::{print_error, setup_signal_handler};

/// 执行单次TCP Ping并返回结果 - 简化超时逻辑
async fn execute_single_ping(
    target: &SocketAddr,
    formatted_host: &str, // 预先格式化，避免重复计算
    timeout: u64,
    seq_num: u32,
    verbose: bool,
    color: bool,
) -> (bool, Option<Duration>) {
    let start = Instant::now();
    let timeout_duration = Duration::from_millis(timeout);

    let result = tcp_connect(target, timeout).await;
    let elapsed = start.elapsed();

    // 简化超时检查 - 直接使用Duration比较，更高效
    if elapsed >= timeout_duration {
        let error_msg = format!("从 {} 超时: seq={}", formatted_host, seq_num);

        if color {
            println!("{}", error_msg.red());
        } else {
            println!("{}", error_msg);
        }

        if verbose {
            println!(
                "  -> 超时详情: 响应时间 {:.2}ms 超过超时阈值 {}ms",
                elapsed.as_secs_f64() * 1000.0,
                timeout
            );
        }

        return (false, None);
    }

    match result {
        Ok(local_addr) => {
            let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
            let success_msg = format!(
                "从 {} 收到响应: seq={} time={:.2}ms",
                formatted_host, seq_num, elapsed_ms
            );

            if color {
                println!("{}", success_msg.green());
            } else {
                println!("{}", success_msg);
            }

            if verbose {
                if let Some(addr) = local_addr {
                    println!("  -> 本地连接详情: {} -> {}", addr, target);
                } else {
                    println!("  -> 无法获取本地连接信息");
                }
            }

            (true, Some(elapsed))
        }
        Err(err) => {
            // 简化错误处理 - 移除重复的超时检查
            let error_msg = if err.contains("timed out") || err.contains("超时") {
                format!("从 {} 超时: seq={}", formatted_host, seq_num)
            } else {
                format!("从 {} 无法连接: seq={}", formatted_host, seq_num)
            };

            if color {
                println!("{}", error_msg.red());
            } else {
                println!("{}", error_msg);
            }

            if verbose {
                println!("  -> 连接失败详情: {}", err);
            }

            (false, None)
        }
    }
}

/// 执行TCP Ping循环并收集统计数据 - 优化控制流和字符串处理
async fn ping_host(ip: std::net::IpAddr, args: &Args, running: Arc<AtomicBool>) -> PingStats {
    let mut stats = PingStats::new();
    let target = SocketAddr::new(ip, args.port);

    // 预先格式化主机端口字符串，避免在循环中重复计算
    let formatted_host = if ip.is_ipv6() {
        format!("[{}]:{}", ip, args.port)
    } else {
        format!("{}:{}", args.host, args.port)
    };

    println!(
        "正在对 {} ({} - {}) 端口 {} 执行 TCP Ping",
        args.host,
        if ip.is_ipv4() { "IPv4" } else { "IPv6" },
        ip,
        args.port
    );

    if args.verbose {
        println!(
            "测试参数: 超时={} ms, 间隔={} ms, 测试次数={}",
            args.timeout,
            args.interval,
            if args.count == 0 {
                "无限".to_string()
            } else {
                args.count.to_string()
            }
        );
    }

    let mut seq = 0;
    let interval_duration = Duration::from_millis(args.interval);

    while running.load(Ordering::Relaxed) && (args.count == 0 || seq < args.count) {
        let (success, duration) = execute_single_ping(
            &target,
            &formatted_host, // 传递预先格式化的字符串
            args.timeout,
            seq,
            args.verbose,
            args.color,
        )
        .await;

        stats.update(success, duration);
        seq += 1;

        if !running.load(Ordering::Relaxed) || (args.count > 0 && seq >= args.count) {
            break;
        }

        tokio::time::sleep(interval_duration).await;
    }

    stats
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // 解析主机名到IP地址
    let filtered_ips = match resolve_host(&args.host, args.ipv4, args.ipv6, args.verbose) {
        Ok(ips) => ips,
        Err(e) => {
            print_error(&e, args.color);
            return Ok(());
        }
    };

    let ip = filtered_ips[0]; // 已确保至少有一个IP

    // 设置信号处理
    let running = Arc::new(AtomicBool::new(true));
    setup_signal_handler(running.clone());

    // 执行TCP Ping
    let stats = ping_host(ip, &args, running).await;

    // 打印最终统计信息
    if stats.transmitted > 0 {
        stats.print_summary(&args.host, args.verbose);
    }

    Ok(())
}
