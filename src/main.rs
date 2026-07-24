#[allow(unused_imports)]
use std::io::{self, Write};
use std::{io::Read, print};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap(); // uses flush to ensure the prompt is displayed before reading input
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        let command = command.trim();

        if command == "exit" {
            break;
        }

        if command.starts_with("echo") {
            println!("{}", &command[5..]);
        } else {
            println!("{}: command not found", command);
        }
    }
}
