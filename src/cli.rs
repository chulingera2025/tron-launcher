use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tronctl")]
#[command(about = "Tron FullNode 编排器", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 初始化 Tron FullNode 环境
    Init {
        /// 快照类型: none, lite, full (不提供则交互式选择)
        #[arg(short, long)]
        snapshot: Option<String>,

        /// FullNode 版本 (默认最新)
        #[arg(short, long)]
        version: Option<String>,

        /// 跳过环境检查
        #[arg(long)]
        skip_checks: bool,
    },

    /// 启动 Tron FullNode
    Start {
        /// 后台运行
        #[arg(short, long)]
        daemon: bool,
    },

    /// 停止 Tron FullNode
    Stop {
        /// 强制停止
        #[arg(short, long)]
        force: bool,
    },

    /// 重启 Tron FullNode
    Restart {
        /// 后台运行
        #[arg(short, long)]
        daemon: bool,
    },

    /// 查看 Tron FullNode 状态
    Status {
        /// 详细输出
        #[arg(short, long)]
        verbose: bool,
    },

    /// 查看 Tron FullNode 日志
    Logs {
        /// 跟随日志输出
        #[arg(short, long)]
        follow: bool,

        /// 显示最后 N 行
        #[arg(short, long, default_value = "100")]
        lines: usize,
    },

    /// 清理 tronctl 产生的所有文件
    Clean {
        /// 跳过确认提示
        #[arg(short = 'y', long)]
        yes: bool,
    },
}
