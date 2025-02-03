use clap::{Parser, Subcommand};
use std::process::exit;

#[derive(Parser)]
#[command(name = "kvs")]
#[command(version=env!("CARGO_PKG_VERSION"))]
#[command(author=env!("CARGO_PKG_AUTHORS"))]
#[command(about=env!("CARGO_PKG_DESCRIPTION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Set the value of a string key to a string")]
    Set {
        #[arg(value_name = "KEY", required = true, help = "A string key")]
        key: String,

        #[arg(
            value_name = "VALUE",
            required = true,
            help = "The string value of the key"
        )]
        value: String,
    },

    #[command(about = "Get the string value of a given string key")]
    Get {
        #[arg(value_name = "KEY", required = true, help = "A string key")]
        key: String,
    },

    #[command(about = "Remove a given key")]
    Remove {
        #[arg(value_name = "KEY", required = true, help = "A string key")]
        key: String,
    },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Set { key: _, value: _ } => {
            eprintln!("unimplemented");
            exit(1);
        }

        Commands::Get { key: _ } => {
            eprintln!("unimplemented");
            exit(1);
        }

        Commands::Remove { key: _ } => {
            eprintln!("unimplemented");
            exit(1);
        }
    }
}
