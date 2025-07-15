use clap::Parser;
use colored::*;
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

/// 通用超时检查函数
fn check_timeout(
    elapsed: Duration,
    timeout: u64,
    formatted_host: &str,
    seq_num: u32,
    verbose: bool,
) -> bool {
    let timeout_duration = Duration::from_millis(timeout);
    if elapsed >= timeout_duration {
        let error_msg = format!("从 {formatted_host} 超时: seq={seq_num}");
        println!("{error_msg}");

        if verbose {
            println!(
                "  -> 超时详情: 响应时间 {:.2}ms 超过超时阈值 {}ms",
                elapsed.as_secs_f64() * 1000.0,
                timeout
            );
        }

        return true;
    }
    false
}

/// 打印彩色信息
fn print_colored_message(message: &str, color_enabled: bool) {
    if color_enabled {
        println!("{message}", message = message.green());
    } else {
        println!("{message}");
    }
}

/// 执行单次TCP Ping并返回结果 - 简化超时逻辑
async fn execute_single_ping(
    target: &SocketAddr,
    formatted_host: &str, // 预先格式化，避免重复计算
    timeout: u64,
    seq_num: u32,
    verbose: bool,
    color_enabled: bool,
) -> (bool, Option<Duration>) {
    let start = Instant::now();
    let result = tcp_connect(target, timeout).await;
    let elapsed = start.elapsed();

    if check_timeout(elapsed, timeout, formatted_host, seq_num, verbose) {
        return (false, None);
    }

    match result {
        Ok(local_addr) => {
            let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
            let success_msg =
                format!("从 {formatted_host} 收到响应: seq={seq_num} time={elapsed_ms:.2}ms");
            print_colored_message(&success_msg, color_enabled);

            if verbose {
                if let Some(addr) = local_addr {
                    println!("  -> 本地连接详情: {addr} -> {target}");
                } else {
                    println!("  -> 无法获取本地连接信息");
                }
            }

            (true, Some(elapsed))
        }
        Err(err) => {
            let error_msg = format!("从 {formatted_host} 无法连接: seq={seq_num}");
            print_colored_message(&error_msg, color_enabled);

            if verbose {
                println!("  -> 连接失败详情: {err}");
            }

            (false, None)
        }
    }
}

/// 执行TCP Ping循环并收集统计数据 - 优化控制流和字符串处理
async fn ping_host(ip: std::net::IpAddr, args: &Args, running: Arc<AtomicBool>) -> PingStats {
    let mut stats = PingStats::new();
    let target = SocketAddr::new(ip, args.port);

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
            &formatted_host,
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

    let filtered_ips = match resolve_host(&args.host, args.ipv4, args.ipv6, args.verbose) {
        Ok(ips) => ips,
        Err(e) => {
            print_error(&e);
            return Ok(());
        }
    };

    let ip = filtered_ips[0];

    let running = Arc::new(AtomicBool::new(true));
    setup_signal_handler(running.clone());

    let stats = ping_host(ip, &args, running).await;

    if stats.transmitted > 0 {
        stats.print_summary(&args.host, args.verbose);
    }

    Ok(())
}
