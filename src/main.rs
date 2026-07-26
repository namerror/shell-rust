use std::fs::metadata;
use std::io::Error;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::path::Path;
use std::{print};
use std::{env, fs};
use std::process::Command;

fn main() {
    let path = env::var("PATH").unwrap();
    let paths: Vec<&str> = path.split(":").collect();
    let mut dir = env::current_dir().unwrap();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap(); // uses flush to ensure the prompt is displayed before reading input
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        let command = command.trim();
        let builtins = ["exit", "echo", "type", "pwd", "cd"];

        let args: Vec<&str> = command.split(' ').collect();

        if command == "exit" {
            break;
        } else if command.starts_with("echo") {
            println!("{}", &command[5..]);
        } else if command.starts_with("type") {

            if builtins.contains(&args[1]) {
                println!("{} is a shell builtin", &args[1]);
            } else {
                match find_executable(&paths, &args[1]) {
                    Ok(v) => println!("{} is {}", &args[1], v),
                    Err(_error) => println!("{}: not found", &args[1]),
                };
            }
        } else if command == "pwd" {
            println!("{}", dir.clone().into_os_string().into_string().unwrap());
        } else if args[0] == "cd" {
            match cd(&args, &mut dir) {
                Ok(()) => (),
                Err(_e) => println!("cd: {}: No such file or directory", &args[1])
            }
        } else if find_executable(&paths, &args[0]).is_ok() {
            Command::new(&args[0]).args(&args[1..]).status().unwrap();
        } else {
            println!("{}: command not found", command);
        }
    }
}

fn find_executable(paths: &Vec<&str>, command: &str) -> Result<String, Error> {
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

fn cd(args: &Vec<&str>, dir: &mut PathBuf) -> Result<(), Error> {
    if args.len() == 1 {
        *dir = env::home_dir().unwrap();
        Ok(())
    } else if args.len() != 2 {
        Err(Error::new(io::ErrorKind::Other, "Wrong args count."))
    } else {
        let abs_path = resolve_path(args[1], dir.to_owned());
        let canonical = fs::canonicalize(abs_path)?;
        if canonical.is_dir() {
            *dir = canonical.clone();
            Ok(())
        } else {
            Err(Error::new(io::ErrorKind::InvalidInput, "Not a dir."))
        }
    }
}

fn resolve_path(rel_path: &str, dir: PathBuf) -> PathBuf {
    if rel_path.starts_with("~") {
        env::home_dir().unwrap().join(rel_path[1..].to_string())
    } else {
        dir.join(rel_path)
    }
} 