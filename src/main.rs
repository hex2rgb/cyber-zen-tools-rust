mod commands;
mod config;

use clap::{Parser, Subcommand};
use config::init_config;

#[derive(Parser)]
#[command(name = "cyber-zen")]
#[command(about = "跨平台命令行工具集，专注于开发工作流优化")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Git 提交并推送
    Gcm {
        /// 提交信息（可选）
        message: Option<String>,
    },
    /// 压缩图片文件
    Compress {
        /// 源文件或文件夹路径
        #[arg(long)]
        src: String,
        /// 目标路径（可选）
        #[arg(long)]
        dist: Option<String>,
        /// 压缩比率 0.1-1.0（可选，默认0.8）
        #[arg(long, default_value_t = 0.8)]
        rate: f64,
    },
    /// 启动静态文件服务器
    Server {
        /// 服务目录（可选，默认当前目录）
        dir: Option<String>,
        /// 端口号（可选，默认3000）
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
    /// 显示工具状态
    Status,
    /// 卸载程序
    Uninstall,
}

fn main() {
    // 初始化配置
    if let Err(e) = init_config() {
        eprintln!("配置初始化失败: {}", e);
        std::process::exit(1);
    }

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Gcm { message } => commands::gcm::run_gcm(message),
        Commands::Compress { src, dist, rate } => commands::compress::run_compress(src, dist, rate),
        Commands::Server { dir, port } => commands::server::run_server(dir, port),
        Commands::Status => commands::status::run_status(),
        Commands::Uninstall => commands::uninstall::run_uninstall(),
    };

    if let Err(e) = result {
        eprintln!("执行失败: {}", e);
        std::process::exit(1);
    }
}

