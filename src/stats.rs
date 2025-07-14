use std::time::Duration;

pub struct PingStats {
    pub transmitted: u32,
    pub received: u32,
    // 移除 total_time，可以从 rtt_values 计算得出
    pub min_time: Option<Duration>,
    pub max_time: Duration,
    // 存储所有RTT值用于计算统计数据
    pub rtt_values: Vec<Duration>,
    // 新增抖动计算字段
    pub jitter: Option<Duration>,
}

impl PingStats {
    pub fn new() -> Self {
        PingStats {
            transmitted: 0,
            received: 0,
            min_time: None,
            max_time: Duration::from_secs(0),
            rtt_values: Vec::new(),
            jitter: None,
        }
    }

    pub fn update(&mut self, success: bool, rtt: Option<Duration>) {
        self.transmitted += 1;

        if success {
            self.received += 1;

            if let Some(time) = rtt {
                // 优化抖动计算逻辑，避免重复计算
                if let Some(last_rtt) = self.rtt_values.last() {
                    let diff = time.abs_diff(*last_rtt);
                    self.jitter = Some(self.jitter.unwrap_or(diff).saturating_add(diff) / 2);
                }

                self.rtt_values.push(time);

                // 更新最小时间和最大时间
                self.min_time = Some(self.min_time.unwrap_or(time).min(time));
                self.max_time = self.max_time.max(time);
            }
        }
    }

    // 计算总时间 - 从 rtt_values 计算
    pub fn total_time(&self) -> Duration {
        self.rtt_values.iter().sum()
    }

    // 简化中位数计算
    pub fn median_time(&self) -> Option<Duration> {
        if self.rtt_values.is_empty() {
            return None;
        }

        let mut sorted_values = self.rtt_values.clone();
        sorted_values.sort();

        let len = sorted_values.len();
        let mid = len / 2;

        if len % 2 == 0 && len >= 2 {
            // 偶数个元素，取中间两个的平均值
            Some((sorted_values[mid - 1] + sorted_values[mid]) / 2)
        } else {
            // 奇数个元素，直接取中间值
            Some(sorted_values[mid])
        }
    }

    // 计算标准差 - 优化数值计算
    pub fn std_deviation(&self) -> Option<f64> {
        let count = self.received as usize;
        if count <= 1 || self.rtt_values.len() <= 1 {
            return None; // 至少需要两个样本计算标准差
        }

        let mean = self.total_time().as_secs_f64() / count as f64;

        let sum_sq_diff = self
            .rtt_values
            .iter()
            .map(|&time| {
                let diff = time.as_secs_f64() - mean;
                diff * diff
            })
            .sum::<f64>();

        Some((sum_sq_diff / (count - 1) as f64).sqrt())
    }

    pub fn print_summary(&self, hostname: &str, verbose: bool) {
        println!("\n--- {hostname} TCP ping 统计 ---");
        println!(
            "已发送 = {}, 已接收 = {}, 丢失 = {} ({:.1}% 丢失)",
            self.transmitted,
            self.received,
            self.transmitted - self.received,
            if self.transmitted > 0 {
                (self.transmitted as f64 - self.received as f64) / self.transmitted as f64 * 100.0
            } else {
                0.0
            }
        );

        if self.received > 0 {
            let avg_time = self.total_time() / self.received;
            let min_time = self.min_time.unwrap_or(Duration::from_secs(0));

            println!(
                "往返时间(RTT): 最小 = {:.2}ms, 最大 = {:.2}ms, 平均 = {:.2}ms",
                min_time.as_secs_f64() * 1000.0,
                self.max_time.as_secs_f64() * 1000.0,
                avg_time.as_secs_f64() * 1000.0
            );

            if verbose && self.received >= 2 {
                if let Some(median) = self.median_time() {
                    println!("中位数(Median) = {:.2}ms", median.as_secs_f64() * 1000.0);
                }

                if let Some(std_dev) = self.std_deviation() {
                    println!("标准差(StdDev) = {:.2}ms", std_dev * 1000.0);
                }

                if let Some(jitter) = self.jitter {
                    println!("抖动(Jitter) = {:.2}ms", jitter.as_secs_f64() * 1000.0);
                }
            }
        }
    }
}
