use std::ffi::CString;
use std::io;
use std::io::prelude::*;
use std::process;

use nix::sys;
use nix::unistd::{execve, fork, ForkResult};

pub fn run() {
    loop {
        prompt();
        let line = read_line();

        let argv = prepare_argv(&line);
        let command = argv[0].clone();
        let env = Vec::new();
        execute(&command, &argv, &env);
    }
}

fn prompt() {
    print!("> ");
    io::stdout().flush().unwrap_or_else(|err| {
        println!("Failed to flush stdout: {:?}", err);
        process::exit(1);
    });
}

fn read_line() -> String {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap_or_else(|err| {
        println!("Failed to read line: {:?}", err);
        process::exit(1);
    });
    line.trim().to_string()
}

fn prepare_argv(line: &String) -> Vec<CString> {
    let args = line.split_whitespace();
    let mut argv = Vec::new();
    for arg in args {
        argv.push(CString::new(arg).unwrap());
    }

    argv
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
