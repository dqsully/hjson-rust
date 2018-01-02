#[macro_use]
extern crate serde_hjson;

use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use serde_hjson::Value;

fn get_content(name: &str) -> io::Result<String> {
    let mut f = try!(File::open(&Path::new(name)));
    let mut buffer = String::new();
    try!(f.read_to_string(&mut buffer));
    Ok(buffer)
}

fn get_test_content(name: &str) -> io::Result<String> {
    let mut p = format!("tests/cases/{}_test.hjson", name);
    if !Path::new(&p).exists() { p = format!("tests/cases/{}_test.json", name); }
    get_content(&p)
}

fn get_result_content(name: &str) -> io::Result<(String,String)> {
    let p1 = format!("tests/cases/{}_result.json", name);
    let p2 = format!("tests/cases/{}_result.hjson", name);
    Ok(( try!(get_content(&p1)), try!(get_content(&p2))))
}

macro_rules! test {
    ($v: ident) => {
        #[test]
        fn $v() {
            let name = stringify!($v);

            let test_content = get_test_content(name).unwrap();
            let data: Value = serde_hjson::from_str(&test_content).unwrap();

            // let (rjson, rhjson) = get_result_content(name).unwrap();
            let actual_hjson = serde_hjson::to_string(&data).unwrap();
            println!("{:?}", data);
            println!("{}", actual_hjson);
            //
            // if rhjson != actual_hjson {
            //     panic!("{:?}\n---hjson expected\n{}\n---hjson actual\n{}\n---\n", name, rhjson, actual_hjson);
            // }
            // TODO later normal json
        }
    }
}

test!(rust);
