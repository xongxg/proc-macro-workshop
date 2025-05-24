use custom_debug::CustomDebug;
use std::fmt::{Debug, Formatter, Pointer};
use std::marker::PhantomData;
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

// #[derive(CustomDebug)]
// struct GeekKindergarten<T> {
//     blog: T,
//     #[debug = "0b{:08b}"]
//     ideawand: i32,
//     com: bool,
// }

// type S = String;
//
// #[derive(CustomDebug)]
// pub struct Field<T> {
//     marker: PhantomData<T>,
//     string: S,
//     #[debug = "0b{:08b}"]
//     bitmask: u8,
// }
//
// fn assert_debug<F: Debug>() {}

pub trait Trait {
    type Value;
}

#[derive(CustomDebug)]
pub struct Field<T: Trait> {
    values: Vec<T::Value>,
    blog: T::Value,
    ideawand: PhantomData<T::Value>,
}

fn assert_debug<F: Debug>() {}

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

    // let g = GeekKindergarten {
    //     blog: "foo",
    //     ideawand: 1,
    //     com: true,
    // };
    // println!("{:#?}", g);

    // Does not implement Debug.
    // struct NotDebug;
    //
    // assert_debug::<PhantomData<NotDebug>>();
    // assert_debug::<Field<NotDebug>>();

    #[derive(Debug)]
struct Id;

    impl Trait for Id {
        type Value = u8;
    }

    assert_debug::<Field<Id>>();
}
