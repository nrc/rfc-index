// TODO
// CLI
//   scan metadata for new
//     tags

use crate::{
    errors::{Error, Result},
    github::{get_merged_rfc_metadata, init_tag_metadata, update_from_pr, UpdateOptions},
    metadata::{
        all_metadata, all_metadata_numbers, delete_metadata, metadata_exists, open_metadata,
        read_tag_metadata, save_metadata, write_tag_metadata, RfcMetadata,
    },
};
use std::{process, str::FromStr};
use structopt::StructOpt;

mod errors;
mod generate;
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
        Command::Scan {
            open,
            merged,
            force,
        } => {
            if open {
                run_scan_open(force);
            }
            if merged {
                run_scan_merged(force);
            }
            if !open && !merged {
                eprintln!("warning: no scan run, use either --merged or --open");
            }
        }
        Command::Stats => run_stats(),
        Command::Generate => run_generate(),
        Command::Query { tag } => run_query(tag),
        Command::Tag {
            numbers,
            add,
            scan,
            init,
        } => run_tag(numbers, add, scan, init),
        Command::Migrate => run_migrate(),
    }
}

/// A utility for building the RFC index website and maintaining its metadata.
#[derive(StructOpt)]
enum Command {
    /// Add new metadata for an RFC.
    Add {
        /// Identify the RFC by number.
        number: u64,
        /// Replace metadata if it already exists.
        #[structopt(short, long)]
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
    /// Scan the RFC repo for metadata.
    Scan {
        /// Scan open RFC PRs.
        #[structopt(long)]
        open: bool,
        /// Scan merged RFCs.
        #[structopt(long)]
        merged: bool,
        /// Replace existing metadata.
        #[structopt(short, long)]
        force: bool,
    },
    /// Emit stats about the metadata
    Stats,
    /// Generate the RFC website.
    Generate,
    /// Query the metadata.
    Query {
        /// Include RFCs which have the given tag. If no tag is specified, include RFCs with no tag.
        #[structopt(long)]
        tag: Option<Option<String>>,
    },
    /// Set/update tags on metadata
    Tag {
        /// Specify RFCs to update, uses all known RFCs if none are specified.
        numbers: Vec<u64>,
        /// Add a tag to the RFCs.
        #[structopt(long)]
        add: Option<String>,
        /// Scan PRs for tags.
        #[structopt(long)]
        scan: Option<Option<TagScanFlags>>,
        /// Initialise tag metadata.
        #[structopt(long)]
        init: bool,
    },
    /// Migrate metadata between versions.
    Migrate,
}

#[derive(StructOpt)]
struct AddFlags {
    #[structopt(long)]
    filename: String,
    #[structopt(long)]
    start_date: String,
    #[structopt(long)]
    feature_name: Option<String>,
    #[structopt(long)]
    issues: Option<String>,
    #[structopt(long)]
    title: Option<String>,
    // TODO tags/teams
}

#[derive(StructOpt)]
struct SetFlags {
    #[structopt(long)]
    filename: Option<String>,
    #[structopt(long)]
    start_date: Option<String>,
    #[structopt(long)]
    feature_name: Option<String>,
    #[structopt(long)]
    issues: Option<String>,
    #[structopt(long)]
    title: Option<String>,
    // TODO tags/teams
}

#[derive(StructOpt)]
struct GetFlags {
    #[structopt(long)]
    filename: bool,
    #[structopt(long)]
    start_date: bool,
    #[structopt(long)]
    feature_name: bool,
    #[structopt(long)]
    issues: bool,
    #[structopt(long)]
    title: bool,
    #[structopt(long)]
    teams: bool,
    #[structopt(long)]
    tags: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum TagScanFlags {
    Default,
    All,
}

impl Default for TagScanFlags {
    fn default() -> TagScanFlags {
        TagScanFlags::Default
    }
}

impl FromStr for TagScanFlags {
    type Err = Error;

    fn from_str(s: &str) -> Result<TagScanFlags> {
        match s {
            "" => Ok(TagScanFlags::Default),
            "all" => Ok(TagScanFlags::All),
            _ => Err(Error::ParseArg(s.to_owned())),
        }
    }
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
    fn unquote(s: String) -> String {
        if (s.starts_with('`') || s.starts_with('"') || s.starts_with('\''))
            && s.ends_with(s.chars().next().unwrap())
        {
            s[1..s.len() - 1].to_owned()
        } else {
            s
        }
    }

    let input = input.trim();
    if input.to_uppercase() == "NA" || input.to_uppercase() == "N/A" {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut buf = String::new();
    let mut quoted = None;
    for c in input.chars() {
        if quoted.is_none()
            && (c == ' '
                || c == '\n'
                || c == '\r'
                || c == ','
                || c == ';'
                || c == '/'
                || &buf == "and")
        {
            if !buf.is_empty() {
                if buf != "and" {
                    result.push(unquote(buf));
                }
                buf = String::new();
            }
        } else if quoted.is_none() && (c == '`' || c == '"' || c == '\'') {
            quoted = Some(c);
            buf.push(c);
        } else if quoted.is_none() && c == '(' {
            quoted = Some(')');
            buf.push(c);
        } else if quoted.is_none() && c == '[' {
            quoted = Some(']');
            buf.push(c);
        } else {
            buf.push(c);

            if let Some(q) = quoted {
                if c == q {
                    quoted = None;
                }
            }
        }
    }

    if !buf.is_empty() && buf != "and" {
        result.push(unquote(buf));
    }

    result
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
    render_vec!(feature_name);
    render_vec!(issues);
    render_opt!(title);
    render_vec!(teams);
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

fn run_scan_open(_force: bool) {
    unimplemented!();
}

fn run_scan_merged(force: bool) {
    match scan_merged(force) {
        Err(e) => {
            eprintln!("Error: {:?}", e);
            process::exit(ExitCode::Other as i32);
        }
        _ => {}
    }
}

fn scan_merged(force: bool) -> Result<()> {
    let gh_data = get_merged_rfc_metadata()?;
    let tag_metadata = read_tag_metadata()?;
    for datum in gh_data {
        let number = datum.number()?;
        if force || metadata_exists(number).is_err() {
            let mut metadata: RfcMetadata = datum.try_into()?;
            update_from_pr(&mut metadata, &tag_metadata, UpdateOptions::all())?;
            save_metadata(&metadata)?;
        }
        // Progress indicator
        eprint!(".");
    }

    Ok(())
}

fn run_stats() {
    let metadata = match all_metadata() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            process::exit(ExitCode::Other as i32);
        }
    };

    let total = metadata.len();
    let title = metadata.iter().filter(|m| m.title.is_some()).count();
    let one_tag = metadata.iter().filter(|m| !m.tags.is_empty()).count();
    let teams = metadata.iter().filter(|m| !m.teams.is_empty()).count();

    println!("Total RFCs indexed: {}", total);
    println!(
        "RFCs with a title: {} ({:.1}%)",
        title,
        (title as f64 / total as f64) * 100.0
    );
    println!(
        "RFCs with at least one team: {} ({:.1}%)",
        teams,
        (teams as f64 / total as f64) * 100.0
    );
    println!(
        "RFCs with at least one tag: {} ({:.1}%)",
        one_tag,
        (one_tag as f64 / total as f64) * 100.0
    );
}

fn run_generate() {
    match generate::generate() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {:?}", e);
            process::exit(ExitCode::Other as i32);
        }
    }
}

fn run_query(tag: Option<Option<String>>) {
    let mut metadata = match all_metadata() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            process::exit(ExitCode::Other as i32);
        }
    };

    if let Some(tag) = tag {
        match tag {
            Some(tag) => {
                metadata = metadata
                    .into_iter()
                    .filter(|d| d.tags.contains(&tag))
                    .collect()
            }
            None => metadata = metadata.into_iter().filter(|d| d.tags.is_empty()).collect(),
        }
    }

    metadata.sort();
    for m in metadata {
        // FIXME data printed should be specified by the query
        print!("{} ", m.number);
    }
    println!();
}

fn run_tag(numbers: Vec<u64>, add: Option<String>, scan: Option<Option<TagScanFlags>>, init: bool) {
    match tag(numbers, add, scan, init) {
        Err(e) => {
            eprintln!("Error: {:?}", e);
            process::exit(ExitCode::Other as i32);
        }
        _ => {}
    }
}

fn tag(
    mut numbers: Vec<u64>,
    add: Option<String>,
    scan: Option<Option<TagScanFlags>>,
    init: bool,
) -> Result<()> {
    if init {
        return tag_init();
    }

    if numbers.is_empty() {
        numbers = all_metadata_numbers()?;
    }

    let scan = scan.map(|s| s.unwrap_or_default());

    let tag_metadata = if scan.is_some() {
        Some(read_tag_metadata()?)
    } else {
        None
    };

    // eprintln!("info: tagging {}", numbers.len());
    for n in numbers {
        let mut metadata = open_metadata(n)?;
        if let Some(add) = &add {
            if !metadata.tags.contains(&add) {
                metadata.tags.push(add.clone());
            }
        }

        if let Some(scan) = scan {
            let opts = UpdateOptions {
                tags: metadata.tags.is_empty() || scan == TagScanFlags::All,
                teams: metadata.teams.is_empty() || scan == TagScanFlags::All,
            };

            update_from_pr(&mut metadata, tag_metadata.as_ref().unwrap(), opts)?;
        }

        save_metadata(&metadata)?;

        // Progress indicator
        eprint!(".");
    }

    Ok(())
}

fn tag_init() -> Result<()> {
    let data = init_tag_metadata()?;
    write_tag_metadata(data)
}

fn run_migrate() {
    let _metadata = match all_metadata() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            process::exit(ExitCode::Other as i32);
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_multiple_() {
        assert_eq!(parse_multiple(""), Vec::<String>::new());
        assert_eq!(parse_multiple(" "), Vec::<String>::new());
        assert_eq!(parse_multiple(", ,, and    , "), Vec::<String>::new());
        assert_eq!(parse_multiple("NA"), Vec::<String>::new());
        assert_eq!(parse_multiple("N/A "), Vec::<String>::new());
        assert_eq!(
            parse_multiple("foo bar"),
            vec!["foo".to_owned(), "bar".to_owned()]
        );
        assert_eq!(
            parse_multiple("foo, bar"),
            vec!["foo".to_owned(), "bar".to_owned()]
        );
        assert_eq!(
            parse_multiple("foo/bar"),
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
            parse_multiple("foo, bar, and `baz`;"),
            vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]
        );
        assert_eq!(
            parse_multiple("\"foo and bar\""),
            vec!["foo and bar".to_owned()]
        );
        assert_eq!(parse_multiple("(foo/bar)"), vec!["(foo/bar)".to_owned()]);
        assert_eq!(
            parse_multiple(
                "[rust-lang/rust#71249](https://github.com/rust-lang/rust/issues/71249)"
            ),
            vec![
                "[rust-lang/rust#71249](https://github.com/rust-lang/rust/issues/71249)".to_owned()
            ]
        );
    }
}
