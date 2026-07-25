use std::fs::metadata;
use std::io::Error;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::{print};
use std::{env, fs};
use std::process::Command;

fn main() {
    loop {
        let path = env::var("PATH").unwrap();
        let paths: Vec<&str> = path.split(":").collect();
        print!("$ ");
        io::stdout().flush().unwrap(); // uses flush to ensure the prompt is displayed before reading input
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        let command = command.trim();
        let builtins = ["exit", "echo", "type"];

        let args: Vec<&str> = command.split(' ').collect();

        if command == "exit" {
            break;
        } else if command.starts_with("echo") {
            println!("{}", &command[5..]);
        } else if command.starts_with("type") {

            if builtins.contains(&args[1]) {
                println!("{} is a shell builtin", &args[1]);
            } else {
                match find_executable(paths, &args[1]) {
                    Ok(v) => println!("{} is {}", &args[1], v),
                    Err(_error) => println!("{}: not found", &args[1]),
                };
            }
        } else if find_executable(paths, &args[0]).is_ok() {
            Command::new(&args[0]).args(&args[1..]).status().unwrap();
        } else {
            println!("{}: command not found", command);
        }
    }
}

fn find_executable(paths: Vec<&str>, command: &str) -> Result<String, Error> {
    for path in paths {
        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            if path.is_file() && path.file_name().is_some_and(|x| x==command) {
                let metadata = metadata(&path)?;
                if (metadata.permissions().mode() & 0o111) != 0 {
                    return match path.into_os_string().into_string() {
                        Ok(s) => Ok(s),
                        Err(_s) => Err(Error::new(io::ErrorKind::Other, "Failed"))
                    }
                }
            }
        }
    }

    Err(Error::new(io::ErrorKind::Other, "Failed"))
}