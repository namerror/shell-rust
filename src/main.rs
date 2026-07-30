use std::env::{self};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::print;
use std::process::Command;

use crate::utils::{Job, find_executable};

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
    let builtins = ["exit", "echo", "type", "pwd", "cd", "jobs"];
    let mut dir = env::current_dir().unwrap();

    let mut jobs: Vec<Job> = Vec::new();
    let mut job_id_counter: u32 = 1;
    let mut children: Vec<std::process::Child> = Vec::new(); // store the child processes

    let mut args = env::args();
    args.next(); // skip the first argument (the program name)

    // background execution mode
    if args.next().as_deref() == Some("--background") {
        let values = args.collect::<Vec<String>>();
        execute(&mut values.iter().map(|s| s.as_str()).collect(), paths.clone(), builtins.to_vec(), &mut dir, &mut jobs, job_id_counter)?;
        return Ok(());
    }

    // main loop
    loop {
        print!("$ ");
        io::stdout().flush().unwrap(); // uses flush to ensure the prompt is displayed before reading input
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        let command = command.trim();

        let string_args = utils::parse_args(&command);
        let mut args = string_args
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();

        if args.len() == 0 {
            continue;
        }

        // background job
        if args[args.len() - 1] == "&" && args.len() > 1 {
            args.pop();

            let mut child;

            if find_executable(&paths, args[0]).is_err() {
                child = Command::new(env::current_exe()?)
                    .arg("--background")
                    .args(&args)
                    .spawn()?;
            } else {
                child = Command::new(args[0])
                    .args(&args[1..])
                    .spawn()?;
            }

            let job = Job {
                id: job_id_counter,
                pid: child.id(),
                status: "Running".into(),
                command: args.join(" "),
            };
            jobs.push(job);
            println!("[{}] {}", job_id_counter, child.id());
            children.push(child);
            io::stdout().flush().unwrap();
            job_id_counter += 1;
            continue;
        }

        // try to reap any finished background jobs
        let mut i = 0;
        while i < children.len() {
            match children[i].try_wait()? {
                Some(_status) => {
                    let job = &mut jobs[i];
                    job.status = "Done".into();
                    children.remove(i);
                }
                None => {
                    i += 1;
                }
            }
        }

        execute(&mut args, paths.clone(), builtins.to_vec(), &mut dir, &mut jobs, job_id_counter)?;

    }
}

// this is the function each process will call, also the entry point for background jobs
fn execute(args: &mut Vec<&str>, paths: Vec<&str>, builtins: Vec<&str>, dir: &mut std::path::PathBuf, jobs: &mut Vec<Job>, job_id_counter: u32) -> io::Result<()> {
    let stdout = Stdout::Stdout(io::stdout());
    let stderr = Stderr::Stderr(io::stderr());
    let mut stdout_buf = "".to_string();
    let mut stderr_buf = "".to_string();

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
        "" => return Ok(()),
        "exit" => std::process::exit(0),
        "echo" => commands::echo(&args, &mut stdout_buf),
        "type" => commands::r#type(&args, &paths, &builtins, &mut stdout_buf),
        "pwd" => {
            stdout_buf = format!("{}\n", dir.clone().into_os_string().into_string().unwrap())
        }
        "cd" => match commands::cd(&args, dir) {
            Ok(()) => (),
            Err(_e) => println!("cd: {}: No such file or directory", &args[1]),
        },
        "jobs" => commands::jobs(jobs, &mut stdout_buf, job_id_counter),
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

    Ok(())
}