use std::fs::metadata;
use std::io::Error;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
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

        let string_args = parse_args(&command);
        let args = string_args.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

        match args[0] {
            "" => continue,
            "exit" => break,
            "echo" => handle_echo(&args),
            "type" => handle_type(&args, &paths, &builtins),
            "pwd" => println!("{}", dir.clone().into_os_string().into_string().unwrap()),
            "cd" => match cd(&args, &mut dir) {
                Ok(()) => (),
                Err(_e) => println!("cd: {}: No such file or directory", &args[1])
            },
            _ => {
                if find_executable(&paths, &args[0]).is_ok() {
                    Command::new(&args[0]).args(&args[1..]).status().unwrap();
                } else {
                    println!("{}: command not found", command);
                }
            }
        }
    }
}

fn handle_type(args: &Vec<&str>, paths: &Vec<&str>, builtins: &[&str]) {
    if args.len() != 2 {
        println!("type: wrong args count.");
        return;
    }

    let command = args[1];

    if builtins.contains(&command) {
        println!("{} is a shell builtin", command);
    } else {
        match find_executable(paths, command) {
            Ok(v) => println!("{} is {}", command, v),
            Err(_error) => println!("{}: not found", command),
        };
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

fn parse_args(command: &str) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();

    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut string_buf: String= "".to_owned(); // used to construct current arg
    let mut escaped = false; // if the current character should be escaped

    for c in command.chars() {
        if escaped {
            string_buf.push(c);
            escaped = false;
            continue;
        }
        match c {
            '\'' => {
                if !in_double_quote {
                    in_single_quote = !in_single_quote;
                } else {
                    string_buf.push(c);
                }
            }
            '\\' => {
                if !in_single_quote {
                    escaped = true;
                } else {
                    string_buf.push(c);
                }
            }
            '"' => {
                if !in_single_quote {
                    in_double_quote = !in_double_quote;
                } else {
                    string_buf.push(c);
                }
            }
            ' ' => {
                if in_single_quote || in_double_quote {
                    string_buf.push(c);
                } else if !string_buf.is_empty() {
                    args.push(string_buf.clone());
                    string_buf.clear();
                }
            }
            _ => string_buf.push(c),
        }
    }

    if !string_buf.is_empty() {
        args.push(string_buf.clone());
    }

    return args
}

fn handle_echo(args: &Vec<&str>) {
    if args.len() < 2 {
        println!();
        return;
    }
    let message = args[1..].join(" ");
    println!("{}", message);
}