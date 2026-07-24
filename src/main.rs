use std::{io::Read, print};
#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {

    loop {
        print!("$ ");
        io::stdout().flush().unwrap(); // uses flush to ensure the prompt is displayed before reading input
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        if command.trim() == "exit" {
            break;
        }

        println!("{}: command not found", command.trim());        
    }


}
