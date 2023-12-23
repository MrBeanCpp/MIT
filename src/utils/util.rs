use sha1::{Digest, Sha1};
use std::{
    fs, io,
    io::Write,
    option,
    path::{Path, PathBuf},
};

use crate::models::{commit::Commit, object::Hash, tree::Tree};

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
    fs::create_dir_all(path.parent().unwrap()).unwrap(); // ensure父目录
    let mut file =
        fs::File::create(get_working_dir().unwrap().join(path)).expect(format!("无法创建文件：{:?}", path).as_str());
    if let Some(content) = content {
        file.write(content.as_bytes()).unwrap();
    } else {
        // 写入文件名
        file.write(path.file_name().unwrap().to_str().unwrap().as_bytes()).unwrap();
    }
}

pub fn ensure_no_file(path: &Path) {
    // 以测试目录为根目录，删除文件
    if path.exists() {
        fs::remove_file(get_working_dir().unwrap().join(path)).unwrap();
    }
}

/* tools for mit */
pub fn calc_hash(data: &String) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let hash = hasher.finalize();
    hex::encode(hash)
}

/// 计算文件的hash
pub fn calc_file_hash(path: &Path) -> String {
    let data = fs::read_to_string(path).expect(&format!("无法读取文件：{}", path.display()));
    calc_hash(&data)
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
        println!("fatal: not a mit repository (or any of the parent directories): .mit");
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

/// 检查文件是否在dir内(包括子文件夹)， 若不存在则false
pub fn is_inside_dir(file: &Path, dir: &Path) -> bool {
    if file.exists() {
        let file = get_absolute_path(file);
        file.starts_with(dir)
    } else {
        false
    }
}

/// 检测dir是否是file的父目录 (不论文件是否存在)
pub fn is_parent_dir(file: &Path, dir: &Path) -> bool {
    let file = get_absolute_path(file);
    file.starts_with(dir)
}

/// 检查文件是否在工作区内， 若不存在则false
pub fn is_inside_workdir(file: &Path) -> bool {
    is_inside_dir(file, &get_working_dir().unwrap())
}

/// 检查文件是否在.mit内， 若不存在则false
pub fn is_inside_repo(file: &Path) -> bool {
    is_inside_dir(file, &get_storage_path().unwrap())
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

/// 列出工作区所有文件(包括子文件夹)
pub fn list_workdir_files() -> Vec<PathBuf> {
    if let Ok(files) = list_files(&get_working_dir().unwrap()) {
        files
    } else {
        Vec::new()
    }
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

/// 清除Windows下的绝对路径前缀"\\\\?\\" (由[PathBuf::canonicalize]函数产生)
/// <br><a href="https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file#maximum-path-length-limitation">Windows 系统中的文件路径格式</a>
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

/// 获取绝对路径（相对于目录current_dir） 不论是否存在
pub fn get_absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        /*let abs_path = path.canonicalize().unwrap(); //这一步会统一路径分隔符 //canonicalize()不能处理不存在的文件
        clean_win_abs_path_pre(abs_path)*/
        // 所以决定手动解析相对路径中的../ ./
        let mut abs_path = std::env::current_dir().unwrap(); //cur_dir
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    if !abs_path.pop() {
                        panic!("relative path parse error");
                    }
                }
                std::path::Component::Normal(part) => abs_path.push(part),
                std::path::Component::CurDir => {}
                _ => {}
            }
        }
        abs_path
    }
}
/// 整理输入的路径数组（相对、绝对、文件、目录、甚至包括不存在），返回一个绝对路径的文件数组
pub fn integrate_paths(paths: &Vec<PathBuf>) -> Vec<PathBuf> {
    let mut abs_paths = Vec::new();
    for path in paths {
        let path = get_absolute_path(&path); // 统一转换为绝对路径
        if path.is_dir() {
            // 包括目录下的所有文件(子文件夹)
            let files = list_files(&path).unwrap();
            abs_paths.extend(files);
        } else {
            abs_paths.push(path);
        }
    }
    abs_paths.sort();
    abs_paths.dedup(); // 去重
    abs_paths
}

#[derive(Debug, PartialEq)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Invalid,
}
pub fn check_object_type(hash: Hash) -> ObjectType {
    let path = get_storage_path().unwrap().join("objects").join(hash);
    if path.exists() {
        let data = fs::read_to_string(path).unwrap(); //TODO store::load?
        let result: Result<Commit, serde_json::Error> = serde_json::from_str(&data);
        if result.is_ok() {
            return ObjectType::Commit;
        }
        let result: Result<Tree, serde_json::Error> = serde_json::from_str(&data);
        if result.is_ok() {
            return ObjectType::Tree;
        }
        return ObjectType::Blob;
    }
    ObjectType::Invalid
}

/// 判断hash对应的文件是否是commit
pub fn is_typeof_commit(hash: Hash) -> bool {
    check_object_type(hash) == ObjectType::Commit
}

#[cfg(test)]
mod tests {
    use crate::models::{blob::Blob, index::Index};

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
    fn test_integrate_paths() {
        let mut paths = Vec::new();
        paths.push(PathBuf::from("src/utils"));
        paths.push(PathBuf::from("../test_del.txt"));
        paths.push(PathBuf::from("src/utils/util.rs"));
        // paths.push(PathBuf::from("."));
        let abs_paths = integrate_paths(&paths);
        for path in abs_paths {
            println!("{}", path.display());
        }
    }

    #[test]
    fn test_get_absolute_path() {
        let path = Path::new("mit_test_storage/../src/main.rs");
        let abs_path = get_absolute_path(path);
        println!("{:?}", abs_path);

        let mut cur_dir = std::env::current_dir().unwrap();
        cur_dir.push("mit_test_storage");
        cur_dir.pop();
        cur_dir.push("src/main.rs");
        assert_eq!(abs_path, cur_dir);
    }

    #[test]
    fn test_is_inside_repo() {
        setup_test_with_clean_mit();
        let path = Path::new("../Cargo.toml");
        assert_eq!(is_inside_workdir(path), false);

        let path = Path::new(".mit/HEAD");
        assert_eq!(is_inside_workdir(path), true);
    }

    #[test]
    fn test_format_time() {
        let time = std::time::SystemTime::now();
        let formatted_time = format_time(&time);
        println!("{:?}", time);
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

    #[test]
    fn test_check_object_type() {
        setup_test_with_clean_mit();
        assert_eq!(check_object_type("123".into()), ObjectType::Invalid);
        ensure_test_file(Path::new("test.txt"), Some("test"));
        let hash = Blob::new(get_working_dir().unwrap().join("test.txt").as_path()).get_hash();
        assert_eq!(check_object_type(hash), ObjectType::Blob);
        let mut commit = Commit::new(&Index::new(), vec![], "test".to_string());
        assert_eq!(check_object_type(commit.get_tree_hash()), ObjectType::Tree);
        commit.save();
        assert_eq!(check_object_type(commit.get_hash()), ObjectType::Commit);
    }
}
