use std::env;
use std::fs;
use std::ffi::CString;
use std::io;
use std::io::prelude::*;
use std::process;

use nix::sys;
use nix::unistd;
use nix::unistd::ForkResult;

pub fn run() {
    let path = match env::var("PATH") {
        Ok(value) => value,
        Err(_) => String::from(""),
    };
    let paths: Vec<&str> = path.split(":").collect();

    loop {
        prompt().unwrap_or_else(|err| {
            eprintln!("Failed to display prompt: {:?}", err);
            process::exit(1);
        });

        let line = read_line().unwrap_or_else(|err| {
            eprintln!("Failed to read line: {:?}", err);
            process::exit(1);
        });

        if line.is_empty() {
            println!("");
            continue;
        }

        let mut argv = prepare_argv(&line);
        let mut command = argv[0].clone();
        if is_builtin(&command) {
            execute_builtin(&command, &mut argv);
            continue;
        } else if !line.starts_with("/") {
            command = match lookup_path(&command, &paths) {
                Ok(c) => c,
                Err(_) => {
                    eprintln!("command not found");
                    continue;
                }
            };
        }

        let env: Vec<CString> = Vec::new();
        execute(&command, &argv, &env);
    }
}

fn prompt() -> std::io::Result<()> {
    print!("> ");
    io::stdout().flush()?;
    Ok(())
}

fn read_line() -> std::io::Result<String> {
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

fn prepare_argv(line: &String) -> Vec<CString> {
    let args = line.split_whitespace();
    let mut argv = Vec::new();
    for arg in args {
        argv.push(CString::new(arg).unwrap());
    }

    argv
}

fn lookup_path(command: &CString, paths: &Vec<&str>) -> Result<CString, ()> {
    let command = command.to_str().unwrap();
    let command = "/".to_owned() + command;
    for path in paths {
        let dir = match fs::read_dir(path) {
            Ok(d) => d,
            Err(_) => continue,
        };

        for entry in dir {
            let entry = entry.unwrap();
            let file = entry.path().to_string_lossy().into_owned();
            if file.ends_with(command.as_str()) {
                return Ok(CString::new(file).unwrap());
            }
        }
    }

    Err(())
}

fn execute(command: &CString, argv: &[CString], env: &[CString]) {
    match unistd::fork() {
        Ok(ForkResult::Parent { child }) => {
            match sys::wait::waitpid(child, None) {
                Ok(_) => (),
                Err(_) => println!("wait failed"),
            };
        },
        Ok(ForkResult::Child) => {
            match unistd::execve(&command, &argv, &env) {
                Ok(_) => (),
                Err(e) => println!("{:?}", e),
            };
        },
        Err(_) => panic!("fork failed"),
    };
}

fn is_builtin(command: &CString) -> bool {
    let command = command.to_str().unwrap();
    return command == "cd";
}

fn execute_builtin(command: &CString, argv: &mut[CString]) {
    let builtin = command.to_str().unwrap();
    match builtin {
        "cd" => cd(argv).unwrap_or_else(|_err| {
            ()
        }),
        _    => (),
    }
}

fn cd(argv: &mut[CString]) -> nix::Result<()> {
    if argv.len() > 2 {
        println!("too many arguments");
        return Err(nix::Error::UnsupportedOperation);
    }

    let home = match env::var("HOME") {
        Ok(value) => value,
        Err(_) => String::from(""),
    };

    let directory;
    if argv.len() == 2 {
        directory = argv[1].to_str().unwrap();
    } else {
        directory = home.as_str();
    }

    match unistd::chdir(directory) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("failed to change directory: {:?}", e);
            Ok(())
        }
    }
}
