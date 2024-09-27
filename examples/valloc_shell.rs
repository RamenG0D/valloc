// use std::io::Write;

use std::io::Write;

use valloc::allocator::Allocator;

fn main() {
    let mut v = Allocator::new(4096);

    // a vec that stores the variables
    let mut variables = Vec::new();

    // a little python like language which can just test the allocator
    loop {
        print!(">>> "); std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "exit" { break; }
        // the help command
        if input == "help" {
            println!("Commands:");
            println!("\tlet <var_name> = <value>;");
            println!("\tprint <var_name>;");
            println!("\tfree <var_name>;");
            println!("\tvars - list all variables;");
            println!("\texit - exit the shell;");
            continue;
        }

        let mut tokens = input.split_whitespace().map(|x| x.to_string());
        let command = tokens.next().unwrap();

        // should have the syntax of `a = 10`
        if command == "let" {
            let var_name = tokens.next().unwrap().clone();
            let _ = tokens.next().unwrap(); // skip the `=`
            let value = tokens.next().unwrap();
            // just a quick check for a ';' at the end
            let value = value.trim_end_matches(';').parse::<i32>().unwrap();

            // allocate memory for the variable
            let mut ptr = v.alloc(1).unwrap();

            *ptr = value;

            // store the variable name and value
            variables.push((var_name, ptr));
        } else if command == "print" {
            // debug vars list
            let var_name = tokens.next().unwrap().clone();

            let (_, ptr) = variables.iter().find(|(name, _)| name == &var_name).expect("Variable not found");

            println!("{}", **ptr);
        } else if command == "free" {
            let var_name = tokens.next().unwrap().clone();

            // find the variable (the ptr must not be a reference we need to consume it)
            let (index, _) = variables.iter().enumerate().find(|(_, (name, _))| name == &var_name).unwrap();
            // now we have its index we just need to move its value when we get it
            let (_, ptr) = variables.remove(index);

            // free the variable
            v.free(ptr).unwrap();
        } else if command == "vars" {
            for (name, ptr) in &variables {
                println!("{} = {}", name, **ptr);
            }
        } else if let Some((vname, vvalue)) = variables.iter_mut().find(|(vname, _)| *vname == command) {
            macro_rules! parse {
                ($tokens:expr) => {
                    match $tokens {
                        Some(val) => val,
                        None => {
                            println!("Expected value after '{}'", command);
                            continue;
                        }
                    }
                };
                (i32; $tokens:expr) => {
                    match $tokens {
                        Ok(val) => val,
                        Err(_) => {
                            println!("Expected integer value after '{}'", command);
                            continue;
                        }
                    }
                };
            }

            // remove '=' sign
            let _ = parse!(tokens.next());
            // if found we overwrite the current value with the new value
            let value = parse!(tokens.next());
            // remove the ';' at the end
            let value = value.trim_end_matches(';');

            let value = parse!(i32; value.parse::<i32>());
            **vvalue = value;
            println!("{} = {}", vname, value);
        }
    }
}
