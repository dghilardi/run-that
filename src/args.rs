use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct RunThatCli {
    #[clap(subcommand)]
    pub subcommand: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run Script
    Run(RunArgs),
}

#[derive(Args, Debug)]
pub struct RunArgs {
    #[clap(long, short)]
    pub script: String,
    #[clap(num_args = 0.., required = false)]
    pub script_args: Vec<String>,
}