use custom_debug::CustomDebug;
use std::fmt::{Debug, Formatter, Pointer};
// #[derive(Builder, Debug)]
// pub struct Command {
//     executable: String,
//     #[builder(each = "arg")]
//     args: Vec<String>,
//     #[builder(each = "env")]
//     env: Vec<String>,
//     current_dir: Option<String>,
// }
//
// #[derive(Debug, FieldCounter)]
// struct Example {
//     field1: i32,
//     field2: String,
// }
//
// // #[derive(DefaultNew)]
//
// #[derive(Getters)]
// struct Config {
//     host: String,
//     port: u16,
// }

#[derive(CustomDebug)]
struct GeekKindergarten {
    blog: String,
    #[debug = "0b{:08b}"]
    ideawand: i32,
    com: bool,
}

fn main() {
    // let command = Command::builder()
    //     .executable("cargo".to_owned())
    //     .arg("build".to_owned())
    //     .arg("--release".to_owned())
    //     .build()
    //     .unwrap();
    //
    // assert_eq!(command.executable, "cargo");
    //
    // println!("example field count: {:#?}", Example::field_count());
    //
    // // let config = Config::new();
    // // println!("Config - host: {}, port: {}", config.host, config.port);
    // let config = Config {
    //     host: "localhost".to_owned(),
    //     port: 8000,
    // };
    //
    // println!("{}", config.host())

    // seq!(N in 1..4 {
    //     fn f #N() -> u64 {
    //         N * 2
    //     }
    // });

    let g = GeekKindergarten {
        blog: "foo".into(),
        ideawand: 123,
        com: true,
    };
    println!("{:#?}", g);
}
