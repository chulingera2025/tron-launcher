# tronctl

> Tron FullNode 一键部署与生命周期管理工具

[**中文文档**](./README.zh-CN.md) | [**English**](./README.md)

## 项目简介

**tronctl** 是一个使用 Rust 编写的生产级命令行工具，用于简化 Tron FullNode 的完整生命周期管理。从初始化部署到日常运维，内置安全特性和交互式配置，让节点管理变得轻松高效。

### 核心特性

- **一键部署** - 分钟级完成 Tron FullNode 初始化和部署
- **智能下载** - 自动获取最新 FullNode.jar 和快照数据
- **服务器选择** - 基于网络延迟自动选择最快的快照服务器
- **交互配置** - 通过交互式提示配置 JVM 内存和快照选项
- **安全加固** - 路径遍历防护、文件锁机制、可选 MD5 校验
- **生命周期管理** - 启动、停止、重启、监控节点一应俱全
- **实时监控** - 健康检查、RPC 状态、区块同步跟踪
- **环境验证** - 预检查 Java 版本、内存、磁盘空间

## 系统要求

| 组件 | 要求 |
|------|------|
| **操作系统** | Linux (已在 Arch Linux 测试) |
| **Java** | Java 8 (1.8.x) |
| **内存** | 推荐 32 GB |
| **存储** | 推荐 2.5 TB+ SSD |
| **权限** | 需要 root 权限 |

## 安装

### 从源码编译

```bash
# 克隆仓库
git clone https://github.com/yourusername/tron-launcher.git
cd tron-launcher

# 编译 release 版本
cargo build --release

# 安装到系统（可选）
sudo cp target/release/tronctl /usr/local/bin/
```

## 快速开始

### 1. 初始化节点

运行交互式初始化流程：

```bash
sudo tronctl init
```

向导将引导你完成：
- **快照选择** - 选择 Lite（53~ GB）或 Full（2937~ GB）快照，或从创世块同步
- **MD5 校验** - 可选完整性检查（完整下载）或流式解压（节省空间）
- **JVM 内存** - 根据服务器配置堆内存（官方推荐 32 GB 内存）

非交互模式：

```bash
# 使用 Lite 快照（推荐大多数用户）
sudo tronctl init --snapshot lite

# 使用完整快照（归档节点）
sudo tronctl init --snapshot full

# 从创世块开始（不使用快照）
sudo tronctl init --snapshot none
```

### 2. 启动节点

```bash
# 后台运行（守护进程模式）
sudo tronctl start --daemon

# 前台运行（Ctrl+C 停止）
sudo tronctl start
```

### 3. 查看状态

```bash
# 基本状态
sudo tronctl status

# 详细状态（包含同步验证）
sudo tronctl status --verbose
```

**输出示例：**
```
状态: 运行中
PID: 12345
进程存活: ✓
RPC 响应: ✓
当前区块: 67890123
```

### 4. 查看日志

```bash
# 显示最后 100 行（默认）
sudo tronctl logs

# 实时跟踪日志
sudo tronctl logs -f

# 显示最后 500 行
sudo tronctl logs --lines 500
```

### 5. 停止节点

```bash
# 优雅停止（SIGTERM，30秒超时）
sudo tronctl stop

# 强制停止（SIGKILL）
sudo tronctl stop --force
```

### 6. 重启节点

```bash
sudo tronctl restart --daemon
```

## 配置文件

初始化完成后，配置文件位于：

- **`/etc/tronctl/tronctl.toml`** - tronctl 配置（JVM 设置、快照类型）
- **`/etc/tronctl/tron.conf`** - Tron 节点配置（从官方仓库下载）

编辑这些文件可以自定义节点行为。修改 `tron.conf` 后需要重启节点生效。

## 高级用法

### 指定 FullNode 版本

```bash
sudo tronctl init --snapshot lite --version GreatVoyage-v4.7.4
```

### 跳过环境检查

```bash
sudo tronctl init --skip-checks
```

⚠️ **警告：** 仅在确定环境满足要求时使用。

## 安全特性

- **路径遍历防护** - 快照解压时验证所有 tar 条目路径
- **文件锁机制** - 防止多实例同时启动（PID 文件锁）
- **可选 MD5 校验** - 启用时确保快照完整性
- **无 Unwrap Panic** - 所有错误路径使用 expect/Result 正确处理

## 故障排查

### Java 版本错误

```bash
# 检查 Java 版本
java -version

# 期望输出: openjdk version "1.8.0_xxx" 或 "8.x.x"
```

如需安装 Java 8：
```bash
# Arch Linux
sudo pacman -S jdk8-openjdk
sudo archlinux-java set java-8-openjdk

# Debian/Ubuntu
sudo apt install openjdk-8-jdk
```

### 权限不足

所有命令需要 root 权限：
```bash
sudo tronctl <command>
```

### 节点无法启动

1. 查看日志：`sudo tronctl logs`
2. 验证 Java 进程：`ps aux | grep java`
3. 检查端口占用：`sudo netstat -tlnp | grep 8090`
4. 确认内存充足：`free -h`

### RPC 无响应

节点需要时间初始化（通常启动后 30-60 秒）。使用以下命令监控：
```bash
sudo tronctl status
```

如果 2 分钟后 RPC 仍无响应，检查日志中的错误信息。

## 开发

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/yourusername/tron-launcher.git
cd tron-launcher

# 编译调试版本
cargo build

# 运行测试
cargo test

# 运行 linter
cargo clippy

# 格式化代码
cargo fmt
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 显示输出
cargo test -- --nocapture
```

## 技术栈

- **语言：** Rust 2024 Edition
- **异步运行时：** Tokio
- **CLI 框架：** Clap 4.5
- **HTTP 客户端：** Reqwest（支持流式传输）
- **序列化：** Serde, TOML
- **日志：** Tracing
- **交互界面：** Dialoguer
- **文件锁：** fs2
- **归档处理：** tar, flate2, async-compression

## 贡献

欢迎贡献！请随时提交 Pull Request 或创建 Issue。

## 许可证

本项目采用 Apache License 2.0 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 致谢

- [Tron Protocol](https://tron.network/) - 区块链平台
- [java-tron](https://github.com/tronprotocol/java-tron) - 官方 Tron 节点实现
- [java-tron Releases](https://github.com/tronprotocol/java-tron/releases) - FullNode.jar 下载仓库

## 相关链接

- [Tron 官方网站](https://tron.network/)
- [java-tron GitHub](https://github.com/tronprotocol/java-tron)
- [Tron 开发者中心](https://developers.tron.network/)
- [Tron 文档](https://tron.network/documentation)