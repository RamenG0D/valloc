use std::process::Command;

fn main() -> Result<(), &'static str> {
    println!("Generating Bindings...");
    Command::new("cbindgen").args([
        "--config", "cbindings.toml", 
        "--crate", "valloc", 
        "--output", "valloc.h", 
        "--lang", "c"
    ]).spawn().unwrap().wait().unwrap();
    println!("Binding Generation Succesful!");

    println!("Compiling...");
    let mut compile = Command::new("gcc");
    compile.args([
        "-o", "test", 
        "examples/test.c", 
        "-I.", 
        "-L./target/debug/", 
        "-lvalloc",
    ]).spawn().unwrap().wait().unwrap();
    println!("Compilation Succesful!");

    println!("Running...");
    Command::new("./test").spawn().unwrap().wait().unwrap();
    println!("Run Succesful!");

    println!("Cleaning Up...");
    Command::new("rm").args(["valloc.h", "test"]).spawn().unwrap().wait().unwrap();
    println!("Clean Up Succesful!");

    println!("Finished Exiting...");
    Ok(())
}
