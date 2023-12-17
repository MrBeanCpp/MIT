use sha1::{Sha1, Digest};
use std::fs::File;
use std::io::{Write, BufReader, BufRead, Error};

#[test]
fn test_hash() {
    let mut hasher = Sha1::new();
    hasher.update(String::from("hello world"));
    let result = format!("{:x}", hasher.finalize());
    println!("{}", result);
    println!("{}", mit::utils::util::calc_hash(&String::from("hello world")));
}

#[test]
fn test_write() -> Result<(), Error> {
    let path = "lines.txt";
    //create会截断文件
    let mut output = File::create(path)?; // ? 用于传播错误
    write!(output, "Rust\nWrite\nRead4")?;
    Ok(())
}

#[test]
fn test_read() -> Result<(), Error> {
    let path = "lines.txt";
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