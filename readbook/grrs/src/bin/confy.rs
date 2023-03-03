use confy;
use serde::{Serialize, Deserialize};
use std::io;

#[derive(Debug, Default, Serialize, Deserialize)]
struct MyConfig {
    name: String,
    comfy: bool,
    foo: i64,
}

fn main() -> Result<(), io::Error> {
    // in linux the config will load/save in ~/.config/my_app/my_app.toml
    // determined by crate directories::ProjectDirs
    let cfg: MyConfig = confy::load("my_app")?;
    println!("{:#?}", cfg);
    Ok(())
}

