use crate::{errors::Result, github::get_merged_rfc_data};
use handlebars::Handlebars;
use mdbook::utils::render_markdown;
use serde::Serialize;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

const OUT_DIR: &str = "target/out";
const STATIC_DIR: &str = "static";
const TEMPLATE_DIR: &str = "templates";

const INDEX_TEMPLATE: &str = "index.handlebars";
const RFC_TEMPLATE: &str = "rfc.handlebars";

pub fn generate() -> Result<()> {
    // Make out dir.
    // Ignore errors (might already exist).
    let _ = fs::create_dir(OUT_DIR);

    // Copy static data. (TODO walk subdirectories)
    fs::read_dir(STATIC_DIR)?
        .filter_map(|e| e.ok())
        .filter(|p| !p.file_type().unwrap().is_dir())
        .map(|entry| {
            let mut to = PathBuf::new();
            to.push(OUT_DIR);
            to.push(entry.file_name());
            fs::copy(entry.path(), &to)?;
            Ok(())
        })
        .collect::<Result<()>>()?;

    // Generate pages from templates + RFC data + metadata
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    let mut index_path = PathBuf::new();
    index_path.push(TEMPLATE_DIR);
    index_path.push(INDEX_TEMPLATE);
    handlebars.register_template_file("index", index_path)?;
    let mut rfc_path = PathBuf::new();
    rfc_path.push(TEMPLATE_DIR);
    rfc_path.push(RFC_TEMPLATE);
    handlebars.register_template_file("rfc", rfc_path)?;

    let mut rfc_data = get_merged_rfc_data()?;
    rfc_data.sort();

    let mut elements = Vec::with_capacity(rfc_data.len());

    for rfc in rfc_data {
        let number = format!("{:0>4}", rfc.number);
        // TODO title from metadata (fallback to filename)
        let title = format!("foo");
        let url = format!("{}.html", number);

        // RFC pages
        let rfc_text = render_markdown(&rfc.text, false);
        let html = handlebars.render(
            "rfc",
            &RfcTemplateData {
                number: number.clone(),
                title: title.clone(),
                rfc_text,
            },
        )?;
        let mut dest = PathBuf::new();
        dest.push(OUT_DIR);
        dest.push(&url);
        let mut file = File::create(dest)?;
        file.write_all(html.as_bytes())?;

        let element = IndexElement { number, title, url };
        elements.push(element);
    }

    let html = handlebars.render("index", &IndexTemplateData { rfcs: elements })?;
    let mut dest = PathBuf::new();
    dest.push(OUT_DIR);
    dest.push("index.html");
    let mut file = File::create(dest)?;
    file.write_all(html.as_bytes())?;

    Ok(())
}

#[derive(Serialize, Clone)]
struct IndexTemplateData {
    rfcs: Vec<IndexElement>,
}

#[derive(Serialize, Clone)]
struct IndexElement {
    number: String,
    title: String,
    url: String,
}

#[derive(Serialize, Clone)]
struct RfcTemplateData {
    number: String,
    title: String,
    rfc_text: String,
}
