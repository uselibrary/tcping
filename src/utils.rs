use colored::Colorize;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// 统一处理错误信息的打印
pub fn print_error(message: &str, color: bool) {
    if color {
        eprintln!("{}", message.red());
    } else {
        eprintln!("{}", message);
    }
}

// 改进信号处理逻辑，使用更精确的内存顺序
pub fn setup_signal_handler(running: Arc<AtomicBool>) {
    if let Err(err) = ctrlc::set_handler(move || {
        if running.swap(false, Ordering::Relaxed) {
            println!(); // 只在第一次按Ctrl+C时打印新行
        } else {
            std::process::exit(0); // 连续两次按Ctrl+C直接退出
        }
    }) {
        eprintln!("警告: 无法设置信号处理程序: {}", err);
    }
}
