use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// 统一处理错误信息的打印，支持更多格式化选项
pub fn print_error(message: &str) {
    eprintln!("错误: {message}");
}

// 改进信号处理逻辑，使用更精确的内存顺序
pub fn setup_signal_handler(running: Arc<AtomicBool>) {
    if let Err(err) = ctrlc::set_handler(move || {
        if running.swap(false, Ordering::Relaxed) {
            println!();
        } else {
            std::process::exit(0);
        }
    }) {
        eprintln!("警告: 无法设置信号处理程序: {err}");
    }
}
