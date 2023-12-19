use crate::utils::util;

pub enum Head {
    Detached(String),
    Branch(String),
}

pub fn current_head() -> Head {
    let mut head = util::get_storage_path().unwrap();
    head.push("HEAD");
    let head_content = std::fs::read_to_string(head).expect("HEAD文件损坏");
    if head_content.starts_with("ref: refs/heads/") {
        let branch_name = head_content.trim_start_matches("ref: refs/heads/");
        Head::Branch(branch_name.to_string())
    } else {
        Head::Detached(head_content)
    }
}

pub fn update_branch(branch_name: &String, commit_hash: &String) {
    let mut branch = util::get_storage_path().unwrap();
    branch.push("refs");
    branch.push("heads");
    branch.push(branch_name);
    std::fs::write(branch, commit_hash).expect("无法写入branch");
}

pub fn get_branch_head(branch_name: &String) -> std::option::Option<String> {
    // 返回当前分支的commit hash
    let mut branch = util::get_storage_path().unwrap();
    branch.push("refs");
    branch.push("heads");
    branch.push(branch_name);
    if branch.exists() {
        let commit_hash = std::fs::read_to_string(branch).expect("无法读取branch");
        Some(commit_hash)
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use crate::utils::util;

    #[test]
    fn test_current_head() {
        util::setup_test_with_mit();
        let head = super::current_head();
        assert!(
            match head {
                super::Head::Branch(_) => true,
                _ => false,
            },
            "当前不在分支上"
        );
    }

    #[test]
    fn test_edit_branch() {
        util::setup_test_with_mit();
        let branch_name = "test_branch".to_string() + &rand::random::<u32>().to_string();
        let branch_head = super::get_branch_head(&branch_name);
        assert!(branch_head.is_none());

        let commit_hash = "1234567890".to_string();
        super::update_branch(&branch_name, &commit_hash);
        let branch_head = super::get_branch_head(&branch_name);
        assert!(branch_head.is_some());
        assert!(branch_head.unwrap() == commit_hash);
    }
}
