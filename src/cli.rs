use clap::{Parser, Subcommand};
use mit::commands::init::init;

/// Rust实现的简易版本的Git，用于学习Rust语言
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The subcommand to run.
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 初始化仓库
    Init,
    /// 添加文件到暂存区
    Add {
        /// 要添加的文件
        files: Vec<String>,

        #[clap(short, long)]
        all: bool,
    },
    /// 删除文件
    Rm {
        /// 要删除的文件
        files: Vec<String>,
        /// flag 删除暂存区的文件
        #[clap(long, action)]
        cached: bool,
    },
    /// 提交暂存区的文件
    Commit {
        #[clap(short, long)]
        message: String,

        #[clap(long, action)]
        allow_empty: bool,
    },
}
pub fn handle_command() {
    let cli = Cli::parse();
    match cli.command {
        Command::Init => {
            let _ = init();
        }
        Command::Add { files , all } => {
            if files.contains(&".".to_string()) || all {
                println!("add all files");
            } else {
                println!("add: {:?}, all:{:?}", files, all);
            }
        }
        Command::Rm { files, cached } => {
            println!("rm: {:?}, cached= {}", files, cached);
        }
        Command::Commit {
            message,
            allow_empty,
        } => {
            println!("commit: {:?}, allow empty={:?}", message, allow_empty);
        }
    }
}
