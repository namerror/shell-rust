use crate::utils::Job;
use crate::utils::find_executable;
use crate::utils::resolve_path;
use std::io::{self, Error, Write};
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

pub fn r#type(args: &Vec<&str>, paths: &Vec<&str>, builtins: &[&str], stdout: &mut String) {
    if args.len() != 2 {
        println!("type: wrong args count.");
        return;
    }

    let command = args[1];

    if builtins.contains(&command) {
        *stdout = format!("{} is a shell builtin\n", command);
    } else {
        match find_executable(paths, command) {
            Ok(v) => *stdout = format!("{} is {}\n", command, v),
            Err(_error) => println!("{}: not found", command),
        };
    }
}

pub fn echo(args: &Vec<&str>, stdout: &mut String) {
    if args.len() < 2 {
        return;
    }
    let message = args[1..].join(" ");
    *stdout = format!("{}\n", message);
}

pub fn unknown(args: &Vec<&str>, paths: &Vec<&str>, stdout: &mut String, stderr: &mut String) {
    if find_executable(&paths, &args[0]).is_ok() {
        let output = Command::new(&args[0])
            .args(&args[1..])
            .output()
            .expect("failed to execute command");

        if !output.stderr.is_empty() {
            *stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        }

        *stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    } else {
        *stderr = format!("{}: command not found\n", args[0]);
    }
}

pub fn cd(args: &Vec<&str>, dir: &mut PathBuf) -> Result<(), Error> {
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

pub fn jobs(jobs: &mut Vec<Job>, stdout: &mut String) -> Result<(), Error> {
    // try to reap any finished background jobs
    for job in jobs.iter_mut() {
        if let Some(_status) = job.child.try_wait()? {
            job.status = "Done".to_string();
        }
    }

    let mut output = String::new();
    let mut marker = " ";

    let mut i = 0;
    while i < jobs.len() {
        let job = &jobs[i];
        if i == jobs.len() - 1 {
            marker = "+";
        } else if i == jobs.len() - 2 {
            marker = "-";
        }

        output = output + &format!("[{}]{}  {:<24}{}\n", job.id, marker, job.status, job.command); // output is stored before the job is removed
        if job.status == "Done" {
            jobs.remove(i);
        } else {
            i += 1;
        }
    }
    *stdout = output;

    Ok(())
}
