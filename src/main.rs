// TODO
// CLI
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

use crate::{
    errors::{Error, Result},
    metadata::{delete_metadata, metadata_exists, open_metadata, save_metadata, RfcMetadata},
};
use std::process;
use structopt::StructOpt;

mod errors;
mod github;
mod metadata;

fn main() {
    match Command::from_args() {
        Command::Add {
            number,
            force,
            flags,
        } => run_add(number, force, flags),
        Command::Set { number, flags } => run_set(number, flags),
        Command::Get {
            number,
            verbose,
            flags,
        } => run_get(number, verbose, flags),
        Command::Delete { number } => run_delete(number),
    }
}

/// A utility for building the RFC index website and maintaining its metadata.
#[derive(StructOpt)]
enum Command {
    /// Add new metadata for an RFC.
    Add {
        /// Identify the RFC by number.
        number: u64,
        #[structopt(long)]
        force: bool,
        #[structopt(flatten)]
        flags: AddFlags,
    },
    /// Update metadata for an RFC.
    Set {
        /// Identify the RFC by number.
        number: u64,
        #[structopt(flatten)]
        flags: SetFlags,
    },
    /// Query the metadata of an RFC.
    Get {
        /// Identify the RFC by number.
        number: u64,
        /// Verbose output specifies the field and is escaped. Non-verbose prints only the queried value.
        #[structopt(short, long)]
        verbose: bool,
        #[structopt(flatten)]
        flags: GetFlags,
    },
    /// Delete the metadata of an RFC.
    Delete {
        /// Identify the RFC by number.
        number: u64,
    },
}

#[derive(StructOpt)]
struct AddFlags {
    #[structopt(short, long)]
    filename: String,
    #[structopt(long)]
    start_date: String,
    #[structopt(long)]
    merge_date: Option<String>,
    #[structopt(long)]
    feature_name: Option<String>,
    #[structopt(long)]
    issues: Option<String>,
    #[structopt(short, long)]
    title: Option<String>,
    // TODO tags
}

#[derive(StructOpt)]
struct SetFlags {
    #[structopt(short, long)]
    filename: Option<String>,
    #[structopt(long)]
    start_date: Option<String>,
    #[structopt(long)]
    merge_date: Option<String>,
    #[structopt(long)]
    feature_name: Option<String>,
    #[structopt(long)]
    issues: Option<String>,
    #[structopt(short, long)]
    title: Option<String>,
    // TODO tags
}

#[derive(StructOpt)]
struct GetFlags {
    #[structopt(short, long)]
    filename: bool,
    #[structopt(long)]
    start_date: bool,
    #[structopt(long)]
    merge_date: bool,
    #[structopt(long)]
    feature_name: bool,
    #[structopt(long)]
    issues: bool,
    #[structopt(short, long)]
    title: bool,
    #[structopt(long)]
    tags: bool,
}

#[derive(Debug, Copy, Clone)]
enum ExitCode {
    Other = 1,
    MissingMetadata = 2,
}

fn run_add(number: u64, force: bool, flags: AddFlags) {
    if let Err(e) = add_metadata(number, force, flags) {
        eprintln!("Error: {:?}", e);
        process::exit(ExitCode::Other as i32);
    }
}

fn add_metadata(number: u64, force: bool, flags: AddFlags) -> Result<()> {
    if !force {
        if let Ok(_) = metadata_exists(number) {
            return Err(Error::MetadataAlreadyExists);
        }
    }

    let mut metadata = RfcMetadata::new(number, flags.filename, flags.start_date);

    metadata.merge_date = flags.merge_date;
    if let Some(s) = flags.feature_name {
        metadata.feature_name = parse_multiple(&s);
    }
    if let Some(s) = flags.issues {
        metadata.issues = parse_multiple(&s);
    }
    metadata.title = flags.title;

    save_metadata(&metadata)
}

fn parse_multiple(input: &str) -> Vec<String> {
    let mut issues = Vec::new();
    let mut buf = String::new();
    for c in input.chars() {
        if c == ' ' || c == '\n' || c == '\r' || c == ',' || c == ';' || &buf == "and" {
            if !buf.is_empty() {
                if buf != "and" {
                    issues.push(buf);
                }
                buf = String::new();
            }
        } else {
            buf.push(c)
        }
    }

    if !buf.is_empty() && buf != "and" {
        issues.push(buf);
    }

    issues
}

fn run_set(number: u64, flags: SetFlags) {
    match set_metadata(number, flags) {
        Ok(_) => {}
        Err(Error::FileNotFound) => {
            eprintln!("RFC {} does not have metadata", number);
            process::exit(ExitCode::MissingMetadata as i32);
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            process::exit(ExitCode::Other as i32);
        }
    }
}

fn set_metadata(number: u64, flags: SetFlags) -> Result<()> {
    let mut metadata = open_metadata(number)?;

    if let Some(f) = flags.filename {
        metadata.filename = f;
    }
    if let Some(f) = flags.start_date {
        metadata.start_date = f;
    }
    if let Some(f) = flags.merge_date {
        metadata.merge_date = Some(f);
    }
    if let Some(s) = flags.feature_name {
        metadata.feature_name = parse_multiple(&s);
    }
    if let Some(s) = flags.issues {
        metadata.issues = parse_multiple(&s);
    }
    if let Some(f) = flags.title {
        metadata.title = Some(f);
    }

    save_metadata(&metadata)
}

fn run_get(number: u64, verbose: bool, flags: GetFlags) {
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
            if flags.$field {
                if verbose {
                    println!("{}: {:?}", stringify!($field), metadata.$field);
                } else {
                    println!("{}", metadata.$field);
                }
            }
        };
    }
    macro_rules! render_opt {
        ($field: ident) => {
            if flags.$field {
                match metadata.$field {
                    Some(f) => {
                        if verbose {
                            println!("{}: {:?}", stringify!($field), f);
                        } else {
                            println!("{}", f);
                        }
                    }
                    None => {
                        if verbose {
                            println!("{}:", stringify!($field));
                        }
                    }
                }
            }
        };
    }
    macro_rules! render_vec {
        ($field: ident) => {
            if flags.$field {
                let iter = metadata
                    .$field
                    .iter()
                    .map(|i| {
                        if verbose {
                            // TODO impl Display for Tag and use {}
                            format!("{:?}", i)
                        } else {
                            format!("{:?}", i)
                        }
                    })
                    .intersperse(", ".to_owned());

                if verbose {
                    print!("{}: [", stringify!($field));
                } else {
                    print!("[");
                }
                iter.for_each(|s| print!("{}", s));
                println!("]");
            }
        };
    }

    render!(filename);
    render!(start_date);
    render_opt!(merge_date);
    render_vec!(feature_name);
    render_vec!(issues);
    render_opt!(title);
    render_vec!(tags);
}

fn run_delete(number: u64) {
    match delete_metadata(number) {
        Err(Error::FileNotFound) => {
            eprintln!("RFC {} does not have metadata", number);
            process::exit(ExitCode::MissingMetadata as i32);
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            process::exit(ExitCode::Other as i32);
        }
        _ => {}
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_multiple_() {
        assert_eq!(parse_multiple(""), Vec::<String>::new());
        assert_eq!(parse_multiple(" "), Vec::<String>::new());
        assert_eq!(parse_multiple(", ,, and    , "), Vec::<String>::new());
        assert_eq!(
            parse_multiple("foo bar"),
            vec!["foo".to_owned(), "bar".to_owned()]
        );
        assert_eq!(
            parse_multiple("foo, bar"),
            vec!["foo".to_owned(), "bar".to_owned()]
        );
        assert_eq!(
            parse_multiple("foo;bar"),
            vec!["foo".to_owned(), "bar".to_owned()]
        );
        assert_eq!(
            parse_multiple("foo and bar"),
            vec!["foo".to_owned(), "bar".to_owned()]
        );
        assert_eq!(
            parse_multiple("foo, bar, and baz;"),
            vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]
        );
    }
}
