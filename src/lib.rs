use std::env;
use std::fs;
use std::ffi::CString;
use std::io;
use std::io::prelude::*;
use std::process;

use nix::sys;
use nix::unistd::{execve, fork, ForkResult};

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

        let argv = prepare_argv(&line);
        let mut command = argv[0].clone();
        if !line.starts_with("/") {
            command = match lookup_path(&command, &paths) {
                Ok(c) => c,
                Err(_) => {
                    eprintln!("command not found");
                    continue;
                },
            };
        }

        let env: Vec<CString> = Vec::new();
        println!("command: {:?}", command);
        println!("argv: {:?}", argv);
        println!("env: {:?}", env);
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
    match fork() {
        Ok(ForkResult::Parent { child }) => {
            match sys::wait::waitpid(child, None) {
                Ok(_) => (),
                Err(_) => println!("wait failed"),
            };
        },
        Ok(ForkResult::Child) => {
            match execve(&command, &argv, &env) {
                Ok(_) => (),
                Err(e) => println!("{:?}", e),
            };
        },
        Err(_) => panic!("fork failed"),
    };
}
