include!("../src/allocator.rs");
mod vmem { include!("../src/vmem.rs"); }
mod pointer { include!("../src/pointer.rs"); }

use std::io::Write;

fn main() {
    let mut v = Valloc::new(4096);

    // a vec that stores the variables
    let mut variables = Vec::new();

    // a little python like language which can just test the allocator
    loop {        
        print!(">>> "); std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "exit" { break; }

        let mut tokens = input.split_whitespace().map(|x| x.to_string());
        let command = tokens.next().unwrap();

        // should have the syntax of `a = 10`
        if command == "let" {
            let var_name = tokens.next().unwrap().clone();
            let _ = tokens.next().unwrap(); // skip the `=`
            let value = tokens.next().unwrap().parse::<i32>().unwrap();

            // allocate memory for the variable
            let ptr = v.alloc_type(1).unwrap();
            v.write(&ptr, value).unwrap();

            // store the variable name and value
            variables.push((var_name, ptr));
        } else if command == "print" {
            // debug vars list
            let var_name = tokens.next().unwrap().clone();

            let var = variables.iter().find(|(name, _)| name == &var_name).unwrap();

            let value = v.read(&var.1).unwrap();
            println!("{}", value);
        } else if command == "free" {
            let var_name = tokens.next().unwrap().clone();
            let var = variables.iter_mut().find(|(name, _)| name == &var_name).unwrap();
            v.free(&mut var.1).unwrap();

            // remove the variable from the list
            variables.retain(|(name, _)| name != &var_name);
        } else if command == "vars" {
            for (name, ptr) in &variables {
                let value = v.read(&ptr).unwrap();
                println!("{} = {}", name, value);
            }
        } else if let Some((vname, vvalue)) = variables.iter().find(|(vname, _)| *vname == command) {
            // remove '=' sign
            let _ = tokens.next().unwrap();
            // if found we overwrite the current value with the new value
            let value = tokens.next().unwrap();

            let value = value.parse::<i32>().unwrap();
            v.write(&vvalue, value).unwrap();
            println!("{} = {}", vname, value);
        }
    }
}
