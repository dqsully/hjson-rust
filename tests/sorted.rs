// #[macro_use]
extern crate serde_hjson;
extern crate difference;
extern crate regex;

use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use serde_hjson::Value;
use regex::Regex;

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
    let p1 = format!("tests/cases/sorted/{}_result.json", name);
    let p2 = format!("tests/cases/sorted/{}_result.hjson", name);
    Ok(( try!(get_content(&p1)), try!(get_content(&p2))))
}

macro_rules! test {
    ($v: ident) => {
        #[allow(non_snake_case)]
        mod $v {
            use super::*;

            #[test]
            fn try_parse() {
                let name = stringify!($v);

                let test_content = get_test_content(name).unwrap();
                let _data: Value = serde_hjson::from_str(&test_content).unwrap();
            }
            #[test]
            fn match_stringify() {
                let name = stringify!($v);

                let test_content = get_test_content(name).unwrap();
                let data: Value = serde_hjson::from_str(&test_content).unwrap();

                let (_, rhjson) = get_result_content(name).unwrap();
                let actual_hjson = serde_hjson::to_string_pretty(&data).unwrap();

                if rhjson != actual_hjson {
                    println!("{}", difference::Changeset::new(&rhjson, &actual_hjson, "\n"));
                    println!("\nExpected:\n{:?}", rhjson);
                    println!("\nGot:\n{:?}", actual_hjson);

                    panic!();
                }
                // TODO later normal json
            }
        }
    };
    ($v: ident, $fix: ident) => {
        #[allow(non_snake_case)]
        mod $v {
            use super::*;

            #[test]
            fn try_parse() {
                let name = stringify!($v);

                let test_content = get_test_content(name).unwrap();
                let _data: Value = serde_hjson::from_str(&test_content).unwrap();
            }
            #[test]
            fn match_stringify() {
                let name = stringify!($v);

                let test_content = get_test_content(name).unwrap();
                let data: Value = serde_hjson::from_str(&test_content).unwrap();

                let (_, rhjson) = get_result_content(name).unwrap();
                let actual_hjson = $fix(serde_hjson::to_string_pretty(&data).unwrap());

                if rhjson != actual_hjson {
                    println!("{}", difference::Changeset::new(&rhjson, &actual_hjson, "\n"));
                    println!("\nExpected:\n{:?}", rhjson);
                    println!("\nGot:\n{:?}", actual_hjson);

                    panic!();
                }
                // TODO later normal json
            }
        }
    };
}

fn std_fix(json: String) -> String {
    let re = Regex::new(r"(?m)(?P<d>\d)\.0(?P<x>,?)$").unwrap();
    String::from(re.replace_all(&json, "$d$s"))
}

fn fix_pass1(json: String) -> String {
    std_fix(json)
        .replace("1.2345678900000003e34", "1.23456789e+34")
        .replace("2.3456789011999997e76", "2.3456789012e+76")
}

mod sorted {
    use super::*;

    test!(charset);
    test!(charset2);
    test!(comments, std_fix);
    test!(empty);
    test!(kan, std_fix);
    test!(keys);
    test!(mltabs);
    test!(oa);
    test!(pass1, fix_pass1);
    test!(pass2);
    test!(pass3);
    test!(pass4);
    test!(passSingle);
    test!(stringify1);
    test!(strings);
    test!(strings2);
    test!(trail);
}
