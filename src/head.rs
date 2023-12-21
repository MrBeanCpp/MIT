use crate::{models::object::Hash, utils::util};

pub enum Head {
    Detached(String),
    Branch(Hash), // TODO Hash
}

pub fn current_head() -> Head {
    let mut head = util::get_storage_path().unwrap();
    head.push("HEAD");
    let head_content = std::fs::read_to_string(head)
        .expect("HEAD文件损坏")
        .trim_end()
        .to_string(); //去除末尾\n
    if head_content.starts_with("ref: refs/heads/") {
        let branch_name = head_content.trim_start_matches("ref: refs/heads/");
        Head::Branch(branch_name.to_string())
    } else {
        Head::Detached(head_content)
    }
}
fn update_branch_head(branch_name: &String, commit_hash: &String) {
    // 更新分支head
    let mut branch = util::get_storage_path().unwrap();
    branch.push("refs");
    branch.push("heads");
    branch.push(branch_name);
    std::fs::write(&branch, commit_hash).expect(&format!(
        "无法写入branch in {:?} with {}",
        branch, commit_hash
    ));
}

fn get_branch_head(branch_name: &String) -> String {
    // 返回当前分支的commit hash
    let mut branch = util::get_storage_path().unwrap();
    branch.push("refs");
    branch.push("heads");
    branch.push(branch_name);
    if branch.exists() {
        let commit_hash = std::fs::read_to_string(branch).expect("无法读取branch");
        commit_hash
    } else {
        "".to_string() // 分支不存在或者没有commit
    }
}

/**返回当前head指向的commit hash，如果是分支，则返回分支的commit hash*/
pub fn current_head_commit() -> String {
    let head = current_head();
    match head {
        Head::Branch(branch_name) => {
            let commit_hash = get_branch_head(&branch_name);
            commit_hash
        }
        Head::Detached(commit_hash) => commit_hash,
    }
}

/**  将当前的head指向commit_hash，根据当前的head类型，更新不同的文件 */
pub fn update_head_commit(commit_hash: &String) {
    let head = current_head();
    match head {
        Head::Branch(branch_name) => {
            update_branch_head(&branch_name, commit_hash);
        }
        Head::Detached(_) => {
            let mut head = util::get_storage_path().unwrap();
            head.push("HEAD");
            std::fs::write(head, commit_hash).expect("无法写入HEAD");
        }
    }
}

/** 列出本地的branch */
pub fn list_local_branches() -> Vec<String> {
    let mut branches = Vec::new();
    let mut branch_dir = util::get_storage_path().unwrap();
    branch_dir.push("refs");
    branch_dir.push("heads");
    if branch_dir.exists() {
        let entries = std::fs::read_dir(branch_dir).expect("无法读取branch");
        for entry in entries {
            let entry = entry.unwrap();
            let branch_name = entry.file_name().into_string().unwrap();
            branches.push(branch_name);
        }
    }
    branches
}

/** 切换head到branch */
pub fn change_head_to_branch(branch_name: &String) {
    let mut head = util::get_storage_path().unwrap();
    head.push("HEAD");
    let branch_head = get_branch_head(branch_name);
    std::fs::write(head, format!("ref: refs/heads/{}", branch_name)).expect("无法写入HEAD");
    update_head_commit(&branch_head);
}

/** 切换head到非branchcommit */
pub fn change_head_to_commit(commit_hash: &String) {
    let mut head = util::get_storage_path().unwrap();
    head.push("HEAD");
    std::fs::write(head, commit_hash).expect("无法写入HEAD");
}

#[cfg(test)]
mod test {
    use crate::{head::update_branch_head, utils::util};

    #[test]
    fn test_edit_branch() {
        util::setup_test_with_mit();
        let branch_name = "test_branch".to_string() + &rand::random::<u32>().to_string();
        let branch_head = super::get_branch_head(&branch_name);
        assert!(branch_head.is_empty());

        let commit_hash = "1234567890".to_string();
        super::update_branch_head(&branch_name, &commit_hash);
        let branch_head = super::get_branch_head(&branch_name);
        assert!(!branch_head.is_empty());
        assert!(branch_head == commit_hash);
    }

    #[test]
    fn test_list_local_branches() {
        util::setup_test_with_mit();
        let branch_one = "test_branch".to_string() + &rand::random::<u32>().to_string();
        let branch_two = "test_branch".to_string() + &rand::random::<u32>().to_string();
        update_branch_head(&branch_one, &"1234567890".to_string());
        update_branch_head(&branch_two, &"1234567890".to_string());

        let branches = super::list_local_branches();
        assert!(branches.contains(&branch_one));
        assert!(branches.contains(&branch_two));
    }

    #[test]
    fn test_change_head_to_branch() {
        util::setup_test_with_mit();
        let branch_name = "test_branch".to_string() + &rand::random::<u32>().to_string();
        update_branch_head(&branch_name, &"1234567890".to_string());
        super::change_head_to_branch(&branch_name);
        assert!(
            match super::current_head() {
                super::Head::Branch(head_commit) => head_commit == branch_name,
                _ => false,
            },
            "当前不在分支上"
        );
    }

    #[test]
    fn test_change_head_to_commit() {
        util::setup_test_with_mit();
        let commit_hash = "1234567890".to_string();
        super::change_head_to_commit(&commit_hash);
        assert!(
            match super::current_head() {
                super::Head::Detached(head_commit) => head_commit == commit_hash,
                _ => false,
            },
            "当前不在分支上"
        );
    }

    #[test]
    fn test_update_branch_head() {
        util::setup_test_with_mit();
        let branch_name = "test_branch".to_string() + &rand::random::<u32>().to_string();
        let commit_hash = "1234567890".to_string();
        super::update_branch_head(&branch_name, &commit_hash);
        let branch_head = super::get_branch_head(&branch_name);
        assert!(!branch_head.is_empty());
        assert!(branch_head == commit_hash);
    }
}
