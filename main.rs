use customer_model::{DefaultNew, FieldCounter, Getters};
use derive_builder::Builder;
use std::fmt::Debug;

#[derive(Builder, Debug)]
pub struct Command {
    executable: String,
    #[builder(each = "arg")]
    args: Vec<String>,
    #[builder(each = "env")]
    env: Vec<String>,
    current_dir: Option<String>,
}

#[derive(Debug, FieldCounter)]
struct Example {
    field1: i32,
    field2: String,
}

// #[derive(DefaultNew)]

#[derive(Getters)]
struct Config {
    host: String,
    port: u16,
}

fn main() {
    let command = Command::builder()
        .executable("cargo".to_owned())
        .arg("build".to_owned())
        .arg("--release".to_owned())
        .build()
        .unwrap();

    assert_eq!(command.executable, "cargo");

    println!("example field count: {:#?}", Example::field_count());

    // let config = Config::new();
    // println!("Config - host: {}, port: {}", config.host, config.port);
    let config = Config {
        host: "localhost".to_owned(),
        port: 8000,
    };

    println!("{}", config.host())
}
