use mit::utils::util;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Write};

#[test]
fn test_hash() {
    let mut hasher = Sha1::new();
    hasher.update(String::from("hello world"));
    let result = format!("{:x}", hasher.finalize());
    println!("{}", result);
    println!("{}", util::calc_hash(&String::from("hello world")));
}

#[test]
fn test_write() -> Result<(), Error> {
    util::setup_test_with_mit();
    let path = "lines.txt";
    //create会截断文件
    let mut output = File::create(path)?; // ? 用于传播错误
    write!(output, "Rust\nWrite\nRead4")?;
    Ok(())
}

#[test]
fn test_read() -> Result<(), Error> {
    util::setup_test_with_mit();
    let path = "lines.txt";
    util::ensure_test_file(path.as_ref(), None);
    let input = File::open(path)?;
    let buffered = BufReader::new(input);

    for line in buffered.lines() {
        println!("{}", line?);
    }
    Ok(())
}

#[test]
fn test_string() {
    let mut s = String::from("Hello");
    s.push_str(", world!");
    s += "2";
    s.push('!');
    println!("{}", s);
}
