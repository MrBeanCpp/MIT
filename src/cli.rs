use clap::{ArgGroup, Parser, Subcommand};
use mit::commands::{
    add::add, branch::branch, commit::commit, init::init, log::log, remove::remove, status::status, switch::switch,
};

/// Rust实现的简易版本的Git，用于学习Rust语言
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The subcommand to run.
    #[clap(subcommand)]
    command: Command,
}
/// @see <a href="https://juejin.cn/post/7242623208825110586">Rust Clap库学习 - 掘金</a>
#[derive(Subcommand)]
enum Command {
    /// 初始化仓库
    Init,
    /// 添加文件到暂存区
    /// @see <a href="https://juejin.cn/post/7053831273277554696">git add .，git add -A，git add -u，git add * 的区别与联系</a>
    Add {
        /// 要添加的文件
        files: Vec<String>,

        /// 将工作区中所有的文件改动提交至暂存区（包括新增、修改和删除）
        #[clap(short = 'A', long)]
        all: bool,

        /// 将工作区中已跟踪的文件(tracked)更新到暂存区（修改 & 删除）；But不包含新增
        #[clap(short, long)]
        update: bool,
    },
    /// 删除文件
    Rm {
        /// 要删除的文件
        files: Vec<String>,
        /// flag 删除暂存区的文件
        #[clap(long, action)]
        cached: bool,
        /// flag 递归删除目录
        #[clap(short, long)]
        recursive: bool,
    },
    /// 提交暂存区的文件
    Commit {
        #[clap(short, long)]
        message: String,

        #[clap(long, action)]
        allow_empty: bool,
    },
    /// 查看当前状态
    Status,
    /// log 现实提交历史
    #[clap(group = ArgGroup::new("sub").required(false))]
    Log {
        #[clap(short = 'A', long)]
        all: bool,

        #[clap(short, long)]
        number: Option<usize>,
    },
    /// branch
    Branch {
        /// 新分支名
        #[clap(group = "sub")]
        new_branch: Option<String>,

        /// 基于某个commit创建分支
        #[clap(requires = "new_branch")]
        commit_hash: Option<String>,

        /// 列出所有分支
        #[clap(short, long, action, group = "sub", default_value = "true")]
        list: bool,

        /// 删除制定分支，不能删除当前所在分支
        #[clap(short = 'D', long, group = "sub")]
        delete: Option<String>,

        /// 显示当前分支
        #[clap(long, action, group = "sub")]
        show_current: bool,
    },

    /// 切换分支
    Switch {
        /// 要切换的分支
        #[clap(required_unless_present("create"))]
        branch: Option<String>,

        /// 创建并切换到新分支
        #[clap(long, short)]
        create: Option<String>,
    },
    /// restore
    Restore {
        // TODO 行为不确定
        /// 要恢复的文件
        #[clap(required = true)]
        files: Vec<String>,

        /// source
        #[clap(long, short)]
        source: Option<String>,
    },
}
pub fn handle_command() {
    let cli = Cli::parse();
    match cli.command {
        Command::Init => {
            init().expect("初始化失败");
        }
        Command::Add { files, all, update } => {
            add(files, all, update);
        }
        Command::Rm { files, cached, recursive } => {
            remove(files, cached, recursive).expect("删除失败");
        }
        Command::Commit { message, allow_empty } => {
            commit(message, allow_empty);
        }
        Command::Status => {
            status();
        }
        Command::Log { all, number } => {
            log(all, number);
        }
        Command::Branch { list, delete, new_branch, commit_hash, show_current } => {
            branch(new_branch, commit_hash, list, delete, show_current);
        }

        Command::Switch { branch, create } => {
            switch(branch, create);
        }
        Command::Restore { files, source } => {
            println!("files: {:?}, source: {:?}", files, source);
        }
    }
}
