#[cfg(feature = "C")] use std::process::Command;

#[cfg(feature = "C")]
fn create_c_bindings() {
    println!("Checking for `cbindgen`...");
    if let Err(std::io::ErrorKind::NotFound) = Command::new("cbindgen").arg("--version").spawn().map_err(|e| e.kind()) {
        eprintln!("Failed to Find `cbindgen` is it installed?");
        println!("It can be installed with \"cargo install cbindgen\"");
        std::process::exit(1);
    } else {
        println!("`cbindgen` Found!");
    }

    println!("Generating Bindings...");
    Command::new("cbindgen").args([
        "-o", "./valloc.h", 
        "--config", "cbindings.toml", 
        "--crate", "valloc", 
        "--lang", "c"
    ]).spawn().unwrap().wait().expect("Failed to generate bindings");
    println!("Binding Generation Succesful!");
}

fn main() {
    #[cfg(feature = "C")] create_c_bindings();
}
