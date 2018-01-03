// #[macro_use]
extern crate serde_hjson;

use std::fs::File;
// use std::fs;
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
                let actual_hjson = serde_hjson::to_string(&data).unwrap();

                if rhjson != actual_hjson {
                    panic!("{:?}\n---hjson expected\n{}\n---hjson actual\n{}\n---\n", name, rhjson, actual_hjson);
                }
                // TODO later normal json
            }
        }
    }
}
macro_rules! test_failure {
    ($v: ident) => {
        #[allow(non_snake_case)]
            mod $v {
            use super::*;

            #[test]
            #[should_panic]
            fn try_parse() {
                let name = stringify!($v);

                let test_content = get_test_content(name).unwrap();
                let _data: Value = serde_hjson::from_str(&test_content).unwrap();
            }
        }
    }
}

test!(charset);
test!(charset2);
test!(comments);
test!(empty);
test_failure!(Charset1);
test_failure!(JSON02);
test_failure!(JSON05);
test_failure!(JSON07);
test_failure!(JSON10);
test_failure!(JSON11);
test_failure!(JSON12);
test_failure!(JSON13);
test_failure!(JSON14);
test_failure!(JSON15);
test_failure!(JSON16);
test_failure!(JSON17);
test_failure!(JSON19);
test_failure!(JSON20);
test_failure!(JSON21);
test_failure!(JSON22);
test_failure!(JSON23);
test_failure!(JSON26);
test_failure!(JSON28);
test_failure!(JSON29);
test_failure!(JSON30);
test_failure!(JSON31);
test_failure!(JSON32);
test_failure!(JSON33);
test_failure!(JSON34);
test_failure!(Key1);
test_failure!(Key2);
test_failure!(Key3);
test_failure!(Key4);
test_failure!(Key5);
test_failure!(MLStr1);
test_failure!(Obj1);
test_failure!(Obj2);
test_failure!(Obj3);
test_failure!(Str1a);
test_failure!(Str1b);
test_failure!(Str1c);
test_failure!(Str1d);
test_failure!(Str2a);
test_failure!(Str2b);
test_failure!(Str2c);
test_failure!(Str2d);
test_failure!(Str3a);
test_failure!(Str3b);
test_failure!(Str3c);
test_failure!(Str3d);
test_failure!(Str4a);
test_failure!(Str4b);
test_failure!(Str4c);
test_failure!(Str4d);
test_failure!(Str5a);
test_failure!(Str5b);
test_failure!(Str5c);
test_failure!(Str5d);
test_failure!(Str6a);
test_failure!(Str6b);
test_failure!(Str6c);
test_failure!(Str6d);
test_failure!(Str7a);
test_failure!(Str8a);
test!(kan);
test!(keys);
test!(mltabs);
test!(oa);
test!(pass1);
test!(pass2);
test!(pass3);
test!(pass4);
test!(passSingle);
test!(stringify1);
test!(strings);
test!(strings2);
test!(trail);
