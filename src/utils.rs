use colored::Colorize;
use std::net::IpAddr;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

/// 添加一个辅助函数来格式化主机地址和端口
pub fn format_host_port(host: &str, port: u16) -> String {
    // 检查主机名是否是IPv6地址
    if let Ok(addr) = host.parse::<IpAddr>() {
        if addr.is_ipv6() {
            return format!("[{}]:{}", host, port);
        }
    }
    format!("{}:{}", host, port)
}

/// 统一处理错误信息的打印
pub fn print_error(message: &str, color: bool) {
    if color {
        eprintln!("{}", message.red());
    } else {
        eprintln!("{}", message);
    }
}

// 改进信号处理逻辑，更加简洁
pub fn setup_signal_handler(running: Arc<AtomicBool>) {
    ctrlc::set_handler(move || {
        if running.swap(false, Ordering::SeqCst) {
            println!(); // 只在第一次按Ctrl+C时打印新行
        } else {
            std::process::exit(0); // 连续两次按Ctrl+C直接退出
        }
    }).expect("无法设置信号处理程序");
}