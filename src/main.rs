use crate::{errors::Error, metadata::open_metadata};
use std::process;
use structopt::StructOpt;

mod errors;
mod github;
mod metadata;

// TODO
// CLI
//   add/edit/delete metadata
//   get status - exists, merged, get a value
//   add/remove tag
//   scan metadata for new
//   update from scan
//   check
//     number == filename
//     tag typos
//     compare with scan
//     date formats
//     subcommands - check for missing title, tags
//   generate website (handlebars)
fn main() {
    match Command::from_args() {
        Command::Get {
            number,
            verbose,
            filename,
            start_date,
        } => run_get(number, verbose, filename, start_date),
        _ => {}
    }
}

// TODO docs
#[derive(StructOpt)]
enum Command {
    Add {
        number: u64,
        filename: String,
        start_date: String,
        // TODO other fields
    },
    Set {
        number: u64,
        #[structopt(short, long)]
        filename: Option<String>,
        #[structopt(long)]
        start_date: Option<String>,
        // TODO other fields
    },
    Get {
        number: u64,
        #[structopt(short, long)]
        verbose: bool,
        #[structopt(short, long)]
        filename: bool,
        #[structopt(long)]
        start_date: bool,
        // TODO other fields
    },
    Delete {
        number: u64,
    },
}

#[derive(Debug, Copy, Clone)]
enum ExitCode {
    Other = 1,
    MissingMetadata = 2,
}

fn run_get(number: u64, verbose: bool, filename: bool, start_date: bool) {
    let metadata = match open_metadata(number) {
        Ok(m) => m,
        Err(Error::FileNotFound) => {
            eprintln!("RFC {} does not have metadata", number);
            process::exit(ExitCode::MissingMetadata as i32);
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            process::exit(ExitCode::Other as i32);
        }
    };

    // Print a single field of metadata
    macro_rules! render {
        ($field: ident) => {
            if $field {
                if verbose {
                    println!("{}: {:?}", stringify!($field), metadata.$field);
                } else {
                    println!("{}", metadata.$field)
                }
            }
        };
    }

    render!(filename);
    render!(start_date);
}
