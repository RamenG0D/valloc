use std::process::Command;

fn main() {
    println!("Generating Bindings...");
    Command::new("cbindgen").args([
        "--config", "cbindings.toml", 
        "--crate", "valloc", 
        "--output", "valloc.h", 
        "--lang", "c"
    ]).spawn().unwrap().wait().unwrap();
    println!("Binding Generation Succesful!");
}
