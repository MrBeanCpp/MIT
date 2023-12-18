use sha1::{Digest, Sha1};

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

pub fn get_storage_path() -> Result<String, std::io::Error> {
    /*递归获取储存库 */
    let mut current_dir = std::env::current_dir()?;
    loop {
        let mut git_path = current_dir.clone();
        git_path.push(".mit");
        if git_path.exists() {
            return Ok(git_path.to_str().unwrap().to_string());
        }
        if !current_dir.pop() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Not a git repository",
            ));
        }
    }
}

pub fn format_time(time: &std::time::SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Utc> = time.clone().into();
    datetime.format("%Y-%m-%d %H:%M:%S.%3f").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_storage_path() {
        let path = get_storage_path();
        match path {
            Ok(path) => println!("{}", path),
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
}