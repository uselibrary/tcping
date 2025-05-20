#!/bin/bash

# 测试目标
TARGETS=(
  "google.com -p 80"
  "github.com -p 443"
  "localhost -p 22"
  "192.168.1.1 -p 80"
  "nonexistent.example.com -p 80"  # 不存在的域名
)

# 基本连通性测试
echo "=== 基本连通性测试 ==="
for target in "${TARGETS[@]}"; do
  echo "测试 $target"
  ./tcping $target -n 5 -c
  echo "----------------------------"
done

# 性能测试（响应时间）
echo -e "\n=== 响应时间测试 ==="
echo "对 google.com:443 进行 100 次测试"
./tcping google.com -p 443 -n 100 | grep -E 'min|max|avg'

# 并发测试
echo -e "\n=== 并发测试 ==="
echo "同时对多个目标进行测试"
for target in "${TARGETS[@]:0:3}"; do
  ./tcping $target -n 10 -c &
done
wait

# 资源消耗测试
echo -e "\n=== 资源消耗测试 ==="
echo "测量 CPU 和内存使用"
time ./tcping google.com -p 80 -n 1000

# 模拟不同网络条件的测试（需要 root 权限）
echo -e "\n=== 模拟网络延迟测试 ==="
if [ "$EUID" -eq 0 ]; then
  echo "添加 100ms 的网络延迟"
  tc qdisc add dev lo root netem delay 100ms
  ./tcping localhost -p 22 -n 10 -c
  echo "清除网络延迟设置"
  tc qdisc del dev lo root
else
  echo "需要 root 权限才能模拟网络延迟"
fi
