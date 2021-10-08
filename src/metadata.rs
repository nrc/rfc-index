use crate::errors::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const METADATA_VERSION: u64 = 1;
const METADATA_DIR: &str = "metadata";

#[derive(Serialize, Deserialize, Debug)]
pub struct RfcMetadata {
    pub version: u64,
    pub number: u64,
    pub filename: String,
    pub start_date: String,
    pub feature_name: Vec<String>,
    pub issues: Vec<String>,
    pub title: Option<String>,
    pub tags: Vec<Tag>,
}

impl RfcMetadata {
    pub fn new(number: u64, filename: String, start_date: String) -> RfcMetadata {
        RfcMetadata {
            version: METADATA_VERSION,
            number,
            filename,
            start_date,
            feature_name: Vec::new(),
            issues: Vec::new(),
            title: None,
            tags: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Tag {
    Team(Team),
    Topic(Topic),
    Custom(String),
    /// Never implemented and not intended to be so.
    Retired,
    Superseded,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum Topic {
    Lang(LangTopic),
    Libs(LibsTopic),
    Core(CoreTopic),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LangTopic {
    Traits,
    TraitObjects,
    Dsts,
    DataTypes,
    Attributes,
    Generics,
    Syntax,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CoreTopic {
    Processes,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LibsTopic {
    Std,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Team {
    Lang,
    Libs,
    Cargo,
    Core,
    Tools,
}

fn metadata_filename(number: u64) -> String {
    format!("{}/{:0>4}.json", METADATA_DIR, number)
}

pub fn save_metadata(metadata: &RfcMetadata) -> Result<()> {
    let serialized = serde_json::to_string(metadata)?;
    let mut file = File::create(metadata_filename(metadata.number))?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

pub fn open_metadata(number: u64) -> Result<RfcMetadata> {
    read_metadata(Path::new(&metadata_filename(number)))
}

fn read_metadata(path: &Path) -> Result<RfcMetadata> {
    let mut file = File::open(path)?;
    let mut serialized = String::new();
    file.read_to_string(&mut serialized)?;
    let result: RfcMetadata = serde_json::from_str(&serialized)?;

    if result.version > METADATA_VERSION {
        Err(Error::UnsupportedMetadataVersion(result.version))
    } else {
        Ok(result)
    }
}

pub fn delete_metadata(number: u64) -> Result<()> {
    fs::remove_file(metadata_filename(number))?;
    Ok(())
}

pub fn metadata_exists(number: u64) -> Result<()> {
    Path::new(&metadata_filename(number))
        .exists()
        .then(|| ())
        .ok_or(Error::FileNotFound)
}

pub fn all_metadata() -> Result<Vec<RfcMetadata>> {
    fs::read_dir(METADATA_DIR)?
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().unwrap().is_dir())
        .map(|e| read_metadata(&e.path()))
        .collect()
}
