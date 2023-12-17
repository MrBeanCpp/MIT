use std::{env, fs, io};

/**
初始化mit仓库 创建.mit/objects .mit/refs/heads .mit/HEAD
并设置 .mit 为隐藏文件夹
无法重复初始化
*/
pub fn init() -> io::Result<()> {
    let dir = env::current_dir()?;
    let mit_dir = dir.join(".mit");
    if mit_dir.exists() {
        println!("!Already a mit repo - [{}]", dir.display());
        return Ok(());
    }

    let dirs = [
        mit_dir.join("objects"),
        mit_dir.join("refs/heads"),
    ];
    // 创建 .git 目录和子目录
    for dir in &dirs {
        fs::create_dir_all(dir)?;
    }
    fs::write(mit_dir.join("HEAD"), "ref: refs/heads/master\n")?;

    set_dir_hidden(&mit_dir.to_str().unwrap())?; // 设置目录隐藏 (跨平台)
    println!("Initialized empty mit repository in {}", dir.display());
    Ok(())
}

#[cfg(target_os = "windows")]
fn set_dir_hidden(dir: &str) -> io::Result<()> {
    use std::process::Command;
    Command::new("attrib")
        .arg("+H")
        .arg(dir)
        .spawn()?
        .wait()?; // 等待命令执行完成
    Ok(())
}

#[cfg( not(target_os = "windows"))]
fn set_dir_hidden(dir: &str) -> io::Result<()> { //类unix系统下'.'开头就已经是隐藏文件(夹)了
    Ok(())
}
