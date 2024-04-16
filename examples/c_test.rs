use std::process::Command;

fn main() {
    println!("Building Library...");
    Command::new("cargo").args(["build", "--release", "--features", "C"]).spawn().unwrap().wait().unwrap();
    println!("Library Build Succesful!");

    println!("Checking for `cbindgen`...");
    while let Err(std::io::ErrorKind::NotFound) = Command::new("cbindgen").arg("--version").spawn().map_err(|e| e.kind()) {
        eprintln!("Failed to Find `cbindgen` is it installed?");
        println!("Do you want to install it? (y/n)");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if input.trim().eq_ignore_ascii_case("y") || input.trim().eq_ignore_ascii_case("yes") {
            Command::new("cargo").args(["install", "cbindgen"]).spawn().unwrap().wait().unwrap();
        } else {
            std::process::exit(1);
        }
    }
    println!("`cbindgen` Found!");

    println!("Generating Bindings...");
    Command::new("cbindgen").args([
        "--config", "cbindings.toml", 
        "--crate", "valloc", 
        "--output", "valloc.h", 
        "--lang", "c"
    ]).spawn().unwrap().wait().expect("Failed to generate bindings");
    println!("Binding Generation Succesful!");

    println!("Compiling...");
    let mut compile = Command::new("gcc");
    compile.args([
        "-o", "test", 
        "examples/test.c", 
        "-I.", 
        "-L./target/release/", 
        "-lvalloc",
        "-DC_BINDGEN"
    ]).spawn().unwrap().wait().unwrap();
    println!("Compilation Succesful!");

    println!("Running...");
    Command::new("./test").spawn().unwrap().wait().unwrap();
    println!("Run Succesful!");

    println!("Cleaning Up...");
    Command::new("rm").args(["valloc.h", "test"]).spawn().unwrap().wait().unwrap();
    println!("Clean Up Succesful!");

    println!("Finished Exiting...");
}
