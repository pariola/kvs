use clap::{Parser, Subcommand};
use std::{env, process::exit};

use kvs::{KvStore, KvsError, Result};

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

    #[command(name = "rm", about = "Remove a given key")]
    Remove {
        #[arg(value_name = "KEY", required = true, help = "A string key")]
        key: String,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let mut exit_code = 0;

    let mut store = KvStore::open(env::current_dir()?)?;

    match args.command {
        Commands::Get { key } => match store.get(key) {
            Ok(None) => {
                println!("Key not found");
            }

            Ok(Some(value)) => {
                println!("{value}");
            }

            Err(err) => {
                exit_code = -1;
                println!("unhandled err: {:?}", err);
            }
        },

        Commands::Set { key, value } => match store.set(key, value) {
            Ok(_) => {}

            Err(err) => {
                exit_code = -1;
                println!("unhandled err: {:?}", err);
            }
        },

        Commands::Remove { key } => match store.remove(key) {
            Ok(_) => {}

            Err(KvsError::KeyNotFound) => {
                exit_code = -1;
                println!("Key not found");
            }

            Err(err) => {
                exit_code = -1;
                println!("unhandled err: {:?}", err);
            }
        },
    };

    exit(exit_code)
}
