pub fn switch(branch: Option<String>, create: Option<String>) {
    // TODO
    match create {
        Some(branch_name) => match branch {
            Some(branch) => {
                println!("craete and switch to branch: {:?} base on {:?}", branch_name, branch);
            }
            None => {
                println!("create and switch to branch: {:?}", branch_name);
            }
        },
        None => {
            println!("switch to branch: {:?}", branch.unwrap());
        }
    }
}
