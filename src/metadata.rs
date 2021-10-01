use crate::errors::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const METADATA_VERSION: u64 = 1;

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
}

// TODO should be custom?
#[derive(Serialize, Deserialize, Debug)]
pub enum Topic {
    Traits,
    TraitObjects,
    Dsts,
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
    format!("metadata/{:0>4}.json", number)
}

pub fn save_metadata(metadata: &RfcMetadata) -> Result<()> {
    let serialized = serde_json::to_string(metadata)?;
    let mut file = File::create(metadata_filename(metadata.number))?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

pub fn open_metadata(number: u64) -> Result<RfcMetadata> {
    let mut file = File::open(metadata_filename(number))?;
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
