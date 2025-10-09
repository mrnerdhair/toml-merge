use std::{
    fs,
    path::PathBuf,
};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    /// Output as JSON
    #[structopt(short, long)]
    json_out: bool,

    /// Input files. These will be merged in order, so later files take precedence.
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn merge(merged: &mut toml::Value, value: &toml::Value) {
    match value {
        toml::Value::String(_) |
        toml::Value::Integer(_) |
        toml::Value::Float(_) |
        toml::Value::Boolean(_) |
        toml::Value::Datetime(_) => *merged = value.clone(),
        toml::Value::Array(x) => {
            match merged {
                toml::Value::Array(merged) => {
                    for (k, v) in x.iter().enumerate() {
                        match merged.get_mut(k) {
                            Some(x) => merge(x, v),
                            None => {
                                let _ = merged.insert(k.clone(), v.clone());
                            },
                        }
                    }
                },
                _ => *merged = value.clone(),
            }
        },
        toml::Value::Table(x) => {
            match merged {
                toml::Value::Table(merged) => {
                    for (k, v) in x.iter() {
                        match merged.get_mut(k) {
                            Some(x) => merge(x, v),
                            None => {
                                let _ = merged.insert(k.clone(), v.clone());
                            },
                        }
                    }
                },
                _ => *merged = value.clone(),
            }
        },
    }
}

fn toml_to_json(x: toml::Value) -> serde_json::Value {
    match x {
        toml::Value::String(x) => serde_json::Value::String(x),
        toml::Value::Integer(x) => serde_json::Value::Number(x.into()),
        toml::Value::Float(x) => serde_json::Value::Number(serde_json::Number::from_f64(x).unwrap()),
        toml::Value::Boolean(x) => serde_json::Value::Bool(x),
        toml::Value::Datetime(x) => serde_json::Value::String(x.to_string()),
        toml::Value::Array(x) => serde_json::Value::Array(x.into_iter().map(toml_to_json).collect()),
        toml::Value::Table(x) => serde_json::Value::Object(x.into_iter().map(|(k, v)| (k, toml_to_json(v))).collect()),
    }
}

fn main() {
    let opt = Opt::from_args();

    let mut merged: toml::Value = toml::Value::Table(toml::value::Table::new());
    for file in opt.files.iter() {
        let value: toml::value::Table = toml::from_slice(&fs::read(file).expect(&format!("Error reading {:?}", file))).expect(&format!("Expected TOML table in {:?}", file));
        merge(&mut merged, &toml::Value::Table(value));
    }

    if opt.json_out {
        println!("{}", serde_json::to_string(&toml_to_json(merged)).unwrap());
    } else {
        println!("{}", merged.to_string());
    }
}