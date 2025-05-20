use clap::Parser;

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
pub struct Args {
    /// 目标主机名或IP地址
    #[clap(required = true)]
    pub host: String,

    /// 目标端口号
    #[clap(short, long, default_value = "80")]
    pub port: u16,

    /// Ping的次数 (0表示无限次)
    #[clap(short = 'n', long = "count", default_value = "0")]
    pub count: u32,

    /// 每次Ping的超时时间(毫秒)
    #[clap(short, long, default_value = "1000")]
    pub timeout: u64,

    /// 两次Ping之间的间隔时间(毫秒)
    #[clap(short = 'i', long, default_value = "1000")]
    pub interval: u64,

    /// 强制使用IPv4
    #[clap(short = '4', long, conflicts_with = "ipv6")]
    pub ipv4: bool,

    /// 强制使用IPv6
    #[clap(short = '6', long, conflicts_with = "ipv4")]
    pub ipv6: bool,
    
    /// 启用详细输出模式
    #[clap(short, long)]
    pub verbose: bool,
    
    /// 启用彩色输出模式
    #[clap(short = 'c', long = "color")]
    pub color: bool,
}
