use sha1::{Digest, Sha1};
use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

use crate::models::{commit::Commit, object::Hash, tree::Tree};

pub const ROOT_DIR: &str = ".mit";

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
// TODO 拆分为PathExt.rs，并定义为trait，实现PathBuf的扩展方法
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

/// 将contents写入path，若文件不存在则创建，且确保父目录存在
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    fs::create_dir_all(path.as_ref().parent().unwrap())?;
    fs::write(path, contents)
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

/// 检测dir是否是file的父目录 (不论文件是否存在) dir可以是一个文件
pub fn is_parent_dir(file: &Path, dir: &Path) -> bool {
    let file = get_absolute_path(file);
    let dir = get_absolute_path(dir);
    file.starts_with(dir)
}

/// 从字符串角度判断path是否是parent的子路径（不检测存在性) alias: [is_parent_dir]
pub fn is_sub_path(path: &Path, parent: &Path) -> bool {
    is_parent_dir(path, parent)
}

/// 判断是否在paths中（包括子目录），不检查存在性
pub fn include_in_paths<T, U>(path: &Path, paths: U) -> bool
where
    T: AsRef<Path>,
    U: IntoIterator<Item = T>,
{
    for p in paths {
        if is_sub_path(path, p.as_ref()) {
            return true;
        }
    }
    false
}

/// 过滤列表中的元素，对.iter().filter().cloned().collect()的简化
pub fn filter<'a, I, O, T, F>(items: I, pred: F) -> O
where
    T: Clone + 'a,
    I: IntoIterator<Item = &'a T>,
    O: FromIterator<T>,
    F: Fn(&T) -> bool,
{
    //items可以是一个引用
    items.into_iter().filter(|item| pred(item)).cloned().collect::<O>()
}

/// 对列表中的元素应用func，对.iter().map().collect()的简化
pub fn map<'a, I, O, T, F>(items: I, func: F) -> O
where
    T: Clone + 'a,
    I: IntoIterator<Item = &'a T>,
    O: FromIterator<T>,
    F: Fn(&T) -> T,
{
    //items可以是一个引用
    items.into_iter().map(|item| func(item)).collect::<O>()
}

/// 过滤列表中的元素，使其在paths中（包括子目录），不检查存在性
pub fn filter_to_fit_paths<T, O>(items: &Vec<T>, paths: &Vec<T>) -> O
where
    T: AsRef<Path> + Clone,
    O: FromIterator<T> + IntoIterator<Item = T>,
{
    filter::<_, O, _, _>(items, |item| include_in_paths(item.as_ref(), paths))
        .into_iter()
        .collect::<O>()
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

/** 列出子文件夹 */
#[allow(dead_code)]
pub fn list_subdir(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let path = get_absolute_path(path);
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.file_name().unwrap_or_default() != ROOT_DIR {
                files.push(path)
            }
        }
    }
    Ok(files)
}
/** 检查一个dir是否包含.mit（考虑.mit嵌套） */
pub fn include_root_dir(dir: &Path) -> bool {
    // 检查子文件夹是否有ROOT_DIR
    if !dir.is_dir() {
        return false;
    }
    let path = get_absolute_path(dir);
    for sub_path in fs::read_dir(path).unwrap() {
        let sub_path = sub_path.unwrap().path();
        if sub_path.file_name().unwrap() == ROOT_DIR {
            return true;
        }
    }
    return false;
}

/// 级联删除空目录，直到遇到 [工作区根目录 | 当前目录]
pub fn clear_empty_dir(dir: &Path) {
    let mut dir = if dir.is_dir() {
        dir.to_path_buf()
    } else {
        dir.parent().unwrap().to_path_buf()
    };
    // 不能删除工作区根目录 & 当前目录
    while !include_root_dir(&dir) && !is_cur_dir(&dir) {
        if is_empty_dir(&dir) {
            fs::remove_dir(&dir).unwrap();
        } else {
            break; //一旦发现非空目录，停止级联删除
        }
        dir.pop();
    }
}

pub fn is_empty_dir(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }
    fs::read_dir(dir).unwrap().next().is_none()
}

pub fn is_cur_dir(dir: &Path) -> bool {
    get_absolute_path(dir) == cur_dir()
}

pub fn cur_dir() -> PathBuf {
    std::env::current_dir().unwrap() //应该是绝对路径吧
}

/// 列出工作区所有文件(包括子文件夹)
pub fn list_workdir_files() -> Vec<PathBuf> {
    if let Ok(files) = list_files(&get_working_dir().unwrap()) {
        files
    } else {
        Vec::new()
    }
}

/// 获取相对于dir的 规范化 相对路径（不包含../ ./）
pub fn get_relative_path_to_dir(path: &Path, dir: &Path) -> PathBuf {
    // 先统一为绝对路径
    let abs_path = if path.is_relative() {
        get_absolute_path(path)
    } else {
        path.to_path_buf()
    };
    // 要考虑path在dir的上级目录的情况，要输出../../xxx
    let common_dir = get_common_dir(&abs_path, dir);
    let mut rel_path = PathBuf::new();
    let mut _dir = dir.to_path_buf();
    while _dir != common_dir {
        rel_path.push("..");
        _dir.pop();
    }
    rel_path.join(abs_path.strip_prefix(common_dir).unwrap())
}

/// 获取两个路径的公共目录
pub fn get_common_dir(p1: &Path, p2: &Path) -> PathBuf {
    let p1 = get_absolute_path(p1);
    let p2 = get_absolute_path(p2);
    let mut common_dir = PathBuf::new();
    for (c1, c2) in p1.components().zip(p2.components()) {
        if c1 == c2 {
            common_dir.push(c1);
        } else {
            break;
        }
    }
    common_dir
}

/// 获取相较于工作区(Working Dir)的相对路径
pub fn to_workdir_relative_path(path: &Path) -> PathBuf {
    get_relative_path_to_dir(path, &get_working_dir().unwrap())
}

/// 获取相较于当前目录的 规范化 相对路径（不包含../ ./）
pub fn get_relative_path(path: &Path) -> PathBuf {
    get_relative_path_to_dir(path, &cur_dir())
}

/// 获取相较于工作区(Working Dir)的绝对路径
pub fn to_workdir_absolute_path(path: &Path) -> PathBuf {
    get_absolute_path_to_dir(path, &get_working_dir().unwrap())
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

/// 获取绝对路径（相对于目录current_dir） 不论是否存在
pub fn get_absolute_path(path: &Path) -> PathBuf {
    get_absolute_path_to_dir(path, &cur_dir())
}

/// 获取绝对路径（相对于目录dir） 不论是否存在
pub fn get_absolute_path_to_dir(path: &Path, dir: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        //相对路径
        /*let abs_path = path.canonicalize().unwrap(); //这一步会统一路径分隔符 //canonicalize()不能处理不存在的文件
        clean_win_abs_path_pre(abs_path)*/
        // 所以决定手动解析相对路径中的../ ./
        let mut abs_path = dir.to_path_buf();
        // 这里会拆分所有组件，所以会自动统一路径分隔符
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
/// 整理输入的路径数组（相对、绝对、文件、目录），返回一个绝对路径的文件数组（只包含exist）
pub fn integrate_paths(paths: &Vec<PathBuf>) -> HashSet<PathBuf> {
    let mut abs_paths = HashSet::new();
    for path in paths {
        let path = get_absolute_path(&path); // 统一转换为绝对路径
        if path.is_dir() {
            // 包括目录下的所有文件(子文件夹)
            let files = list_files(&path).unwrap();
            abs_paths.extend(files);
        } else {
            abs_paths.insert(path);
        }
    }
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
    use crate::{
        models::{blob::Blob, index::Index},
        utils::{test, util::*},
    };

    #[test]
    fn test_write() {
        test::setup_with_empty_workdir();
        let path = Path::new("src/test/a.txt");
        write(path, "test").unwrap();
        assert!(path.exists());
    }

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
        let path = Path::new("./mit_test_storage/.././src\\main.rs");
        let abs_path = get_absolute_path(path);
        println!("{:?}", abs_path);

        let mut cur_dir = std::env::current_dir().unwrap();
        cur_dir.push("mit_test_storage");
        cur_dir.pop();
        cur_dir.push("src\\main.rs");
        assert_eq!(abs_path, cur_dir); // 只比较组件，不比较分隔符
    }

    #[test]
    fn test_get_relative_path() {
        test::setup_with_clean_mit();
        let path = Path::new("../../src\\main.rs");
        let rel_path = get_relative_path_to_dir(&path, &cur_dir());
        println!("{:?}", rel_path);

        assert_eq!(rel_path, path);
    }

    #[test]
    fn test_to_workdir_absolute_path() {
        test::setup_with_clean_mit();
        let path = Path::new("./src/../main.rs");
        let abs_path = to_workdir_absolute_path(path);
        println!("{:?}", abs_path);

        let mut cur_dir = get_working_dir().unwrap();
        cur_dir.push("main.rs");
        assert_eq!(abs_path, cur_dir);
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
        test::setup_with_clean_mit();
        test::ensure_file(Path::new("test/test.txt"), None);
        test::ensure_file(Path::new("a.txt"), None);
        test::ensure_file(Path::new("b.txt"), None);
        let files = list_files(Path::new("./"));
        match files {
            Ok(files) => {
                for file in files {
                    println!("{}", file.display());
                }
            }
            Err(err) => println!("{}", err),
        }

        assert_eq!(list_files(Path::new(".")).unwrap(), list_files(Path::new("./")).unwrap());
    }

    #[test]
    fn test_check_object_type() {
        test::setup_with_clean_mit();
        assert_eq!(check_object_type("123".into()), ObjectType::Invalid);
        test::ensure_file(Path::new("test.txt"), Some("test"));
        let hash = Blob::new(get_working_dir().unwrap().join("test.txt").as_path()).get_hash();
        assert_eq!(check_object_type(hash), ObjectType::Blob);
        let mut commit = Commit::new(&Index::get_instance(), vec![], "test".to_string());
        assert_eq!(check_object_type(commit.get_tree_hash()), ObjectType::Tree);
        commit.save();
        assert_eq!(check_object_type(commit.get_hash()), ObjectType::Commit);
    }

    #[test]
    fn test_check_root_dir() {
        test::setup_with_clean_mit();
        list_workdir_files().iter().for_each(|f| {
            fs::remove_file(f).unwrap();
        });
        list_subdir(Path::new("./")).unwrap().iter().for_each(|f| {
            if include_root_dir(f) {
                fs::remove_dir_all(f).unwrap();
            }
        });
        assert_eq!(include_root_dir(Path::new("./")), true);
        fs::create_dir("./src").unwrap_or_default();
        assert_eq!(include_root_dir(Path::new("./src")), false);
        fs::create_dir("./src/.mit").unwrap_or_default();
        assert_eq!(include_root_dir(Path::new("./src")), true);
    }
}
