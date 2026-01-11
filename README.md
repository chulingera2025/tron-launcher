# tron-launcher (tronctl)

Tron FullNode 一键部署与生命周期管理工具。

## 特性

- 🚀 一键初始化和部署 Tron FullNode
- 📦 自动下载最新 FullNode.jar 和快照数据
- 🌐 智能选择最快的快照服务器
- 🔄 完整的进程生命周期管理
- 📊 实时健康检查和状态监控
- 🛡️ 环境检查和验证

## 系统要求

- **操作系统**: Linux (推荐 Arch Linux)
- **Java**: Java 8 (1.8) - 严格要求
- **内存**: 推荐 32GB
- **磁盘**: 推荐 2.5TB+ SSD
- **权限**: 需要 root 权限

## 安装

```bash
# 克隆仓库
git clone https://github.com/yourusername/tron-launcher.git
cd tron-launcher

# 编译
cargo build --release

# 安装（可选）
sudo cp target/release/tronctl /usr/local/bin/
```

## 快速开始

### 1. 初始化节点

```bash
# 使用 Lite 快照（53GB，推荐用于测试）
sudo tronctl init --snapshot lite

# 使用完整快照（2937GB，生产环境）
sudo tronctl init --snapshot full

# 不使用快照（从0开始同步）
sudo tronctl init --snapshot none
```

### 2. 启动节点

```bash
# 后台运行
sudo tronctl start --daemon

# 前台运行（按 Ctrl+C 停止）
sudo tronctl start
```

### 3. 查看状态

```bash
# 基本状态
sudo tronctl status

# 详细状态（包含区块同步检查）
sudo tronctl status --verbose
```

### 4. 查看日志

```bash
# 查看最近100行日志
sudo tronctl logs

# 实时跟踪日志
sudo tronctl logs -f

# 查看最近500行
sudo tronctl logs --lines 500
```

### 5. 停止节点

```bash
# 优雅停止
sudo tronctl stop

# 强制停止
sudo tronctl stop --force
```

### 6. 重启节点

```bash
sudo tronctl restart --daemon
```

## 命令详解

### `tronctl init`

初始化 Tron FullNode 环境。

**选项：**
- `-s, --snapshot <TYPE>`: 快照类型 (none/lite/full)，默认 none
- `-v, --version <VERSION>`: 指定 FullNode 版本，默认最新
- `--skip-checks`: 跳过环境检查

**示例：**
```bash
sudo tronctl init --snapshot lite
sudo tronctl init --snapshot full --version GreatVoyage-v4.7.4
```

### `tronctl start`

启动 Tron FullNode。

**选项：**
- `-d, --daemon`: 后台运行

**示例：**
```bash
sudo tronctl start --daemon
```

### `tronctl stop`

停止 Tron FullNode。

**选项：**
- `-f, --force`: 强制停止（SIGKILL）

**示例：**
```bash
sudo tronctl stop
sudo tronctl stop --force
```

### `tronctl restart`

重启 Tron FullNode。

**选项：**
- `-d, --daemon`: 后台运行

### `tronctl status`

查看节点状态。

**选项：**
- `-v, --verbose`: 详细输出（包含区块同步检查）

**输出示例：**
```
状态: 运行中
PID: 12345
进程存活: ✓
RPC 响应: ✓
当前区块: 12345678
```

### `tronctl logs`

查看节点日志。

**选项：**
- `-f, --follow`: 实时跟踪日志
- `-l, --lines <N>`: 显示最后 N 行，默认 100

## 目录结构

```
/var/lib/tronctl/       # 数据目录
├── FullNode.jar        # FullNode JAR 文件
└── data/               # 区块链数据

/etc/tronctl/           # 配置目录
├── tron.conf           # Tron 节点配置
└── tronctl.toml        # tronctl 配置

/var/log/tronctl/       # 日志目录
└── fullnode.log        # 节点日志

/run/tronctl/           # 运行时目录
└── tronctl.pid         # PID 文件
```

## 配置

编辑 `/etc/tronctl/tron.conf` 修改 Tron 节点配置。

编辑 `/etc/tronctl/tronctl.toml` 修改 tronctl 配置。

## 故障排查

### Java 版本错误

```bash
# 检查 Java 版本
java -version

# 应该看到 1.8.x 或 8.x
```

### 权限不足

所有命令都需要 root 权限：

```bash
sudo tronctl <command>
```

### 节点无法启动

1. 检查日志：`sudo tronctl logs`
2. 检查 Java 进程：`ps aux | grep java`
3. 检查端口占用：`sudo netstat -tlnp | grep 8090`

### RPC 不响应

节点启动需要时间，通常需要等待 30-60 秒。使用 `sudo tronctl status` 持续监控。

## 开发

```bash
# 克隆仓库
git clone https://github.com/yourusername/tron-launcher.git
cd tron-launcher

# 开发编译
cargo build

# 运行测试
cargo test

# 运行 clippy 检查
cargo clippy

# 格式化代码
cargo fmt
```

## 技术栈

- **语言**: Rust 2024 Edition
- **异步运行时**: Tokio
- **CLI 框架**: Clap
- **HTTP 客户端**: Reqwest
- **序列化**: Serde, TOML
- **日志**: Tracing

## 许可证

MIT License

## 相关链接

- [Tron 官网](https://tron.network/)
- [java-tron GitHub](https://github.com/tronprotocol/java-tron)
- [Tron 开发者文档](https://developers.tron.network/)
