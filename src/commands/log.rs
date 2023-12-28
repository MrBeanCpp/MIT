use crate::{models::Commit, utils::head};
use colored::Colorize;

const DEFAULT_LOG_NUMBER: usize = 10;

pub fn log(all: bool, number: Option<usize>) {
    println!("log all: {:?}, number: {:?}", all, number);
    let _ = __log(all, number);
}

fn __log(all: bool, number: Option<usize>) -> usize {
    let mut log_count = 0usize;

    let head = head::current_head();
    let mut branch_name: Option<String> = None;
    let mut head_commit = match head {
        head::Head::Branch(_branch_name) => {
            let commit = head::get_branch_head(&_branch_name);
            branch_name = Some(_branch_name.clone());
            if commit.is_empty() {
                println!("当前分支{:?}没有任何提交", _branch_name);
                return 0;
            }
            commit
        }
        head::Head::Detached(commit_hash) => commit_hash,
    };

    let mut number = match number {
        Some(number) => number,
        None => DEFAULT_LOG_NUMBER,
    };

    let mut first = true;
    loop {
        log_count += 1;
        let commit = Commit::load(&head_commit);
        if first {
            // TODO: (HEAD -> ttt, ad2)
            first = false;
            print!("{}{}{}{}", "commit ".yellow(), commit.get_hash().yellow(), "(".yellow(), "HEAD".blue());
            if let Some(ref branch_name) = branch_name {
                print!("{}", format!(" -> {}", branch_name).blue());
            }
            println!("{}", ")".yellow());
        } else {
            println!("{}{}{}{}{}", "commit ".yellow(), head_commit.yellow(), "(".yellow(), "HEAD".blue(), ")".yellow());
        }
        println!("Author: {}", commit.get_author());
        println!("Date:   {}", commit.get_date());
        println!();
        println!("    {}", commit.get_message());
        println!();

        if all == false {
            if number > 1 {
                number -= 1;
            } else {
                break;
            }
        }
        if commit.get_parent_hash().len() == 0 {
            break;
        }
        head_commit = commit.get_parent_hash().first().unwrap().clone();
    }
    log_count
}

#[cfg(test)]
mod test {
    use super::super::super::commands;
    use crate::utils::test_util;
    #[test]
    fn test_log() {
        test_util::setup_test_with_clean_mit();
        assert_eq!(super::__log(false, None), 0);
        commands::commit::commit("test commit 2".into(), true);
        assert_eq!(super::__log(false, Some(1)), 1);
        commands::commit::commit("test commit 3".into(), true);
        assert_eq!(super::__log(false, None), 2);
    }
}
