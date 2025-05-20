use clap::Parser;
use colored::Colorize;
use std::net::{SocketAddr};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};

mod cli;
mod network;
mod stats;
mod utils;

use cli::Args;
use stats::PingStats;
use network::{tcp_connect, resolve_host};
use utils::{format_host_port, print_error, setup_signal_handler};

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