use dns_lookup::lookup_host;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tokio::net::TcpStream;

/// 创建TCP连接并返回连接结果和本地地址信息
pub async fn tcp_connect(
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

/// 提取IP地址筛选逻辑为单独函数
pub fn filter_ip_addresses(ip_addrs: Vec<IpAddr>, ipv4: bool, ipv6: bool, verbose: bool) -> Vec<IpAddr> {
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

/// 解析主机名并返回IP地址列表
pub fn resolve_host(host: &str, ipv4: bool, ipv6: bool, verbose: bool) -> Result<Vec<IpAddr>, String> {
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