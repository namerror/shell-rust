use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::print;

mod commands;
mod utils;

enum Stdout {
    Stdout(io::Stdout),
    File(File),
}

enum Stderr {
    Stderr(io::Stderr),
    File(File),
}

fn main() -> io::Result<()> {
    let path = env::var("PATH").unwrap();
    let paths: Vec<&str> = path.split(":").collect();
    let mut dir = env::current_dir().unwrap();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap(); // uses flush to ensure the prompt is displayed before reading input
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        let command = command.trim();
        let builtins = ["exit", "echo", "type", "pwd", "cd", "jobs"];

        let string_args = utils::parse_args(&command);
        let mut args = string_args
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();

        let mut stdout_buf = "".to_string();
        let mut stderr_buf = "".to_string();
        let stdout = Stdout::Stdout(io::stdout());
        let stderr = Stderr::Stderr(io::stderr());

        if args.len() == 0 {
            continue;
        }

        let stdout = if args.len() >= 3 {
            if args[args.len() - 2] == ">" || args[args.len() - 2] == "1>" || args[args.len() - 2] == ">>" || args[args.len() - 2] == "1>>" {
                let file_path = utils::resolve_path(args[args.len() - 1], dir.clone());
                let file = if args[args.len() - 2] == ">" || args[args.len() - 2] == "1>" {
                    File::create(file_path)?
                } else {
                    OpenOptions::new().create(true).append(true).open(file_path)?
                };
                args.truncate(args.len() - 2); // remove the last two args (">" and the file path)
                Stdout::File(file)
            } else {
                stdout
            }
        } else {
            stdout
        };

        let stderr = if args.len() >= 3 {
            if args[args.len() - 2] == "2>" || args[args.len() - 2] == "2>>" {
                let file_path = utils::resolve_path(args[args.len() - 1], dir.clone());
                let file = if args[args.len() - 2] == "2>" {
                    File::create(file_path)?
                } else {
                    OpenOptions::new().create(true).append(true).open(file_path)?
                };
                args.truncate(args.len() - 2); // remove the last two args ("2>" and the file path)
                Stderr::File(file)
            } else {
                stderr
            }
        } else {
            stderr
        };

        match args[0] {
            "" => continue,
            "exit" => break,
            "echo" => commands::echo(&args, &mut stdout_buf),
            "type" => commands::r#type(&args, &paths, &builtins, &mut stdout_buf),
            "pwd" => {
                stdout_buf = format!("{}\n", dir.clone().into_os_string().into_string().unwrap())
            }
            "cd" => match commands::cd(&args, &mut dir) {
                Ok(()) => (),
                Err(_e) => println!("cd: {}: No such file or directory", &args[1]),
            },
            "jobs" => commands::jobs(),
            _ => commands::unknown(&args, &paths, &mut stdout_buf, &mut stderr_buf),
        };

        if !stdout_buf.is_empty() {
            match stdout {
                Stdout::Stdout(mut _out) => {
                    print!("{}", stdout_buf);
                    io::stdout().flush()?;
                }
                Stdout::File(mut file) => {
                    write!(file, "{}", stdout_buf)?;
                }
            }
        }
        if !stderr_buf.is_empty() {
            match stderr {
                Stderr::Stderr(mut _err) => {
                    eprint!("{}", stderr_buf);
                    io::stderr().flush()?;
                }
                Stderr::File(mut file) => {
                    write!(file, "{}", stderr_buf)?;
                }
            }
        }
    }
    Ok(())
}