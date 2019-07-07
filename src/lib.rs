use std::ffi::CString;
use std::io;
use std::io::prelude::*;

use nix::sys;
use nix::unistd::{execve, fork, ForkResult};

pub fn run() {
    loop {
        print!("> ");
        io::stdout().flush()
            .expect("Failed to flush stdout.");

        let mut line = String::new();
        io::stdin().read_line(&mut line)
            .expect("Failed to read line.");
        let line = line.trim();

        let args = line.split_whitespace();
        let mut argv = Vec::new();
        for arg in args {
            argv.push(CString::new(arg).unwrap());
        }

        let command = argv[0].clone();
        let env = Vec::new();

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
}
