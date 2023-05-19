use clap::Parser;
use crate::args::{Commands, RunThatCli};

mod args;

fn main() {
    check_version();
    let args: RunThatCli = RunThatCli::parse();

    match args.subcommand {
        Commands::Run(run_args) => println!("Run {} -- {:?}", run_args.script, run_args.script_args),
    }
}

fn check_version() {
    use update_informer::Check;
    use update_informer::registry::Crates;

    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let informer = update_informer::new(Crates, name, version);
    if let Some(version) = informer.check_version().ok().flatten()  {
        println!("New version is available: {}", version);
    }
}
