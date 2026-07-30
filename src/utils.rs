use std::fs::metadata;
use std::io;
use std::io::Error;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{env, fs};

pub struct Job {
    pub id: u32,
    pub child: std::process::Child,
    pub status: String,
    pub command: String,
}

pub fn find_executable(paths: &Vec<&str>, command: &str) -> Result<String, Error> {
    for path in paths {
        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            if path.is_file() && path.file_name().is_some_and(|x| x == command) {
                let metadata = metadata(&path)?;
                if (metadata.permissions().mode() & 0o111) != 0 {
                    return match path.into_os_string().into_string() {
                        Ok(s) => Ok(s),
                        Err(_s) => Err(Error::new(io::ErrorKind::Other, "Failed")),
                    };
                }
            }
        }
    }

    Err(Error::new(io::ErrorKind::Other, "Failed"))
}

pub fn resolve_path(rel_path: &str, dir: PathBuf) -> PathBuf {
    if rel_path.starts_with("~") {
        env::home_dir().unwrap().join(rel_path[1..].to_string())
    } else {
        dir.join(rel_path)
    }
}

pub fn parse_args(command: &str) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();

    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut string_buf: String = "".to_owned(); // used to construct current arg
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

    return args;
}
