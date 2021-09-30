use std::{fs::File, io::Read};

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

const PR_URL: &str = "https://github.com/rust-lang/rfcs/pull/";
const TEXT_URL: &str = "https://github.com/rust-lang/rfcs/blob/master/text/";
const METADATA_VERSION: u64 = 1;

#[derive(Serialize, Deserialize, Debug)]
struct RfcMetadata {
    version: u64,
    number: u64,
    filename: String,
    start_date: String,
    merge_date: Option<String>,
    feature_name: Vec<String>,
    issues: Vec<String>,
    title: Option<String>,
    tags: Vec<Tag>,
}

#[derive(Serialize, Deserialize, Debug)]
enum Tag {
    Team(Team),
    Topic(Topic),
    Custom(String),
}

// TODO should be custom?
#[derive(Serialize, Deserialize, Debug)]
enum Topic {
    Traits,
    TraitObjects,
    Dsts,
}

#[derive(Serialize, Deserialize, Debug)]
enum Team {
    Lang,
    Libs,
    Cargo,
    Core,
    Tools,
}

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
        } => run_get(number, filename, start_date, verbose).unwrap(),
        _ => {}
    }
}

#[derive(Debug)]
enum Error {}

type Result<T> = std::result::Result<T, Error>;

fn metadata_filename(number: u64) -> String {
    format!("metadata/{:0>4}.json", number)
}

fn run_get(number: u64, filename: bool, start_date: bool, verbose: bool) -> Result<()> {
    let mut file = File::open(metadata_filename(number)).unwrap();
    let mut serialized = String::new();
    file.read_to_string(&mut serialized).unwrap();
    let metadata: RfcMetadata = serde_json::from_str(&serialized).unwrap();

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
        }
    }

    render!(filename);
    render!(start_date);

    Ok(())
}
