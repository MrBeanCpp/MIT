use sha1::{Digest, Sha1};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io, option};

pub const ROOT_DIR: &str = ".mit";
pub const TEST_DIR: &str = "mit_test_storage"; // 执行测试的储存库

/* tools for test */
fn setup_test_dir() {
    color_backtrace::install(); // colorize backtrace
    let cargo_path = std::env::var("CARGO_MANIFEST_DIR");
    let path: PathBuf = {
        if cargo_path.is_err() {
            // vscode DEBUG test没有CARGO_MANIFEST_DIR宏，手动尝试查找cargo.toml
            let mut path = std::env::current_dir().unwrap();
            loop {
                path.push("Cargo.toml");
                if path.exists() {
                    break;
                }
                if !path.pop() {
                    panic!("找不到CARGO_MANIFEST_DIR");
                }
            }
            path.pop();
            path
        } else {
            PathBuf::from(cargo_path.unwrap())
        }
    };
    let mut path = PathBuf::from(path);
    path.push(TEST_DIR);
    if !path.exists() {
        fs::create_dir(&path).unwrap();
    }
    std::env::set_current_dir(&path).unwrap();
}

pub fn setup_test_with_mit() {
    // 将执行目录切换到测试目录
    setup_test_dir();
    let _ = crate::commands::init::init();
}

/// with 初始化的干净的mit
pub fn setup_test_with_clean_mit() {
    setup_test_without_mit();
    let _ = crate::commands::init::init();
}

pub fn setup_test_without_mit() {
    // 将执行目录切换到测试目录，并清除测试目录下的.mit目录
    setup_test_dir();
    let mut path = std::env::current_dir().unwrap();
    path.push(ROOT_DIR);
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
}

pub fn ensure_test_file(path: &Path, content: option::Option<&str>) {
    // 以测试目录为根目录，创建文件
    let mut file = fs::File::create(get_working_dir().unwrap().join(path))
        .expect(format!("无法创建文件：{:?}", path).as_str());
    if let Some(content) = content {
        file.write(content.as_bytes()).unwrap();
    } else {
        // 写入文件名
        file.write(path.file_name().unwrap().to_str().unwrap().as_bytes())
            .unwrap();
    }
}

/* tools for mit */
pub fn calc_hash(data: &String) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let hash = hasher.finalize();
    hex::encode(hash)
}

pub fn storage_exist() -> bool {
    /*检查是否存在储存库 */
    let rt = get_storage_path();
    match rt {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn check_repo_exist() {
    if !storage_exist() {
        panic!("不是合法的mit仓库");
    }
}

/// 获取.mit目录路径
pub fn get_storage_path() -> Result<PathBuf, io::Error> {
    /*递归获取储存库 */
    let mut current_dir = std::env::current_dir()?;
    loop {
        let mut git_path = current_dir.clone();
        git_path.push(ROOT_DIR);
        if git_path.exists() {
            return Ok(git_path);
        }
        if !current_dir.pop() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{:?} is not a git repository", std::env::current_dir()?),
            ));
        }
    }
}

/// 获取项目工作区目录, 也就是.mit的父目录
pub fn get_working_dir() -> Option<PathBuf> {
    if let Some(path) = get_storage_path().unwrap().parent() {
        Some(path.to_path_buf())
    } else {
        None
    }
}

pub fn format_time(time: &std::time::SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Utc> = time.clone().into();
    datetime.format("%Y-%m-%d %H:%M:%S.%3f").to_string()
}

/// 递归遍历给定目录及其子目录，列出所有文件，除了.mit
pub fn list_files(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if path.is_dir() {
        if path.file_name().unwrap_or_default() == ROOT_DIR {
            // 跳过 .mit 目录
            return Ok(files);
        }
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // 递归遍历子目录
                files.extend(list_files(&path)?);
            } else {
                // 将文件的路径添加到列表中
                files.push(path);
            }
        }
    }
    Ok(files)
}

/// 获取相对于dir的相对路径
pub fn get_relative_path(path: &Path, dir: &Path) -> PathBuf {
    let path = if path.is_relative() {
        get_absolute_path(path)
    } else {
        path.to_path_buf()
    };
    let relative_path = path.strip_prefix(dir).unwrap();
    relative_path.to_path_buf()
}

/// 获取相较于工作区(Working Dir)的相对路径
pub fn to_workdir_relative_path(path: &Path) -> PathBuf {
    get_relative_path(path, &get_working_dir().unwrap())
}

/// 获取相较于工作区(Working Dir)的绝对路径
pub fn to_workdir_absolute_path(path: &Path) -> PathBuf {
    if path.is_relative() {
        get_working_dir().unwrap().join(path)
    } else {
        path.to_path_buf()
    }
}

fn is_executable(path: &str) -> bool {
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    #[cfg(windows)]
    {
        let path = Path::new(path);
        match path.extension().and_then(|s| s.to_str()) {
            Some(ext) => ext.eq_ignore_ascii_case("exe") || ext.eq_ignore_ascii_case("bat"),
            None => false,
        }
    }
}

pub fn get_file_mode(path: &Path) -> String {
    // if is_executable(path.to_str().unwrap()) {
    //     "100755".to_string()
    // } else {
    //     "100644".to_string()
    // }
    if path.is_dir() {
        "40000".to_string() // 目录
    } else if is_executable(path.to_str().unwrap()) {
        "100755".to_string() // 可执行文件
    } else {
        "100644".to_string()
    }
}

/// 清除Windows下的绝对路径前缀"\\\\?\\"
/// <a href="https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file#maximum-path-length-limitation">Windows 系统中的文件路径格式</a>
pub fn clean_win_abs_path_pre(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        const DOS_PREFIX: &str = "\\\\?\\";
        let path_str = path.to_string_lossy();
        if path_str.starts_with(DOS_PREFIX) {
            PathBuf::from(&path_str[DOS_PREFIX.len()..])
        } else {
            path
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        path
    }
}

/// 获取绝对路径（相对于当前current_dir）
pub fn get_absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        let abs_path = path.canonicalize().unwrap(); //这一步会统一路径分隔符
        clean_win_abs_path_pre(abs_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_storage_path() {
        let path = get_storage_path();
        match path {
            Ok(path) => println!("{:?}", path),
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => println!("Not a git repository"),
                _ => assert!(false, "Unexpected error"),
            },
        }
    }

    #[test]
    fn test_format_time() {
        let time = std::time::SystemTime::now();
        let formatted_time = format_time(&time);
        println!("{}", formatted_time);
    }

    #[test]
    fn test_list_files() {
        let files = list_files(Path::new("F:\\Git-Test\\list-test"));
        match files {
            Ok(files) => {
                for file in files {
                    println!("{}", file.display());
                }
            }
            Err(err) => println!("{}", err),
        }
    }
}
