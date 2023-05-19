use anyhow::Context;
use clap::Parser;
use crate::args::{Commands, RunThatCli};
use crate::config::load_config;

mod args;
mod config;

fn main() -> anyhow::Result<()> {
    check_version();
    let args: RunThatCli = RunThatCli::parse();

    match args.subcommand {
        Commands::Run(run_args) => {
            let cwd = run_args.path.or_else(|| std::env::current_dir().ok())
                .context("Cannot load cwd")?;

            let config = load_config(cwd)
                .context("Error parsing config")?;
            println!("Run {} -- {:?}", run_args.script, run_args.script_args)
        },
    }
    Ok(())
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
