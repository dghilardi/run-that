use clap::Parser;
use crate::args::{Commands, RunThatCli};

mod args;

fn main() {
    let args: RunThatCli = RunThatCli::parse();

    match args.subcommand {
        Commands::Run(run_args) => println!("Run {} -- {:?}", run_args.script, run_args.script_args),
    }
}
