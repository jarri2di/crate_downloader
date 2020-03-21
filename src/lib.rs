#![deny(warnings)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate failure;

use failure::Error;
use futures_util::StreamExt;
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

#[derive(Deserialize)]
struct Crate {
    name: String,
    #[serde(rename = "vers")]
    version: String,
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "Rust Crates Downloader",
    about = "Download Rust crates for offline development."
)]
pub struct Opt {
    /// Existing path where the local crates.io-index repo resides
    #[structopt(short, long, env, parse(try_from_str = parse_path))]
    index_path: PathBuf,

    /// Existing path where crate files will be downloaded to
    #[structopt(short, long, env, parse(try_from_str = parse_path))]
    download_path: PathBuf,

    /// Base URL of the crates.io crates endpoint
    #[structopt(default_value = "https://crates.io/api/v1/crates", short, long, env)]
    crates_io_url: String,

    /// Number of lightweight threads to spawn for downloading crates
    #[structopt(default_value = "50", parse(try_from_str = parse_thread_size), short, long, env)]
    threads: u8,
}

// Convenience function to validate paths exist
fn parse_path(s: &str) -> Result<PathBuf, &'static str> {
    if Path::new(&s).exists() {
        Ok(PathBuf::from(&s))
    } else {
        Err("Value must be an existing path")
    }
}

// Convenience function to validate thread size value
fn parse_thread_size(s: &str) -> Result<u8, &'static str> {
    let range: std::ops::Range<u8> = 1..100;
    match s.parse() {
        Ok(n) => {
            if range.contains(&n) {
                Ok(n)
            } else {
                Err("Expected value in range 1-99")
            }
        }
        Err(_) => Err("Expected value in range 1-99"),
    }
}

// Convenience function to generate crate download url
fn generate_crate_url(c: &Crate, base_url: &str) -> String {
    format!("{}/{}/{}/download", base_url, c.name, c.version)
}

// Convenience function to generate local crate path
fn generate_crate_download_path(c: &Crate, path: &Path) -> PathBuf {
    let crate_filename = format!("{}-{}.crate", c.name, c.version);
    path.join(Path::new(&crate_filename))
}

// Convenience function to determine if directory or file is hidden
fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| !s.starts_with('.'))
        .unwrap_or(false)
}

// Convenience function to determine if it's an index config file
fn is_not_config_file(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| !s.eq("config.json"))
        .unwrap_or(false)
}

// Stream crate from crates.io to local disk
async fn stream_to_file(
    client: &reqwest::Client,
    download_url: &str,
    local_crate_path: &Path,
) -> Result<(), Error> {
    let mut stream = client.get(download_url).send().await?.bytes_stream();
    let mut file = File::create(local_crate_path)?;
    while let Some(data) = stream.next().await {
        file.write_all(&data?)?;
    }

    Ok(())
}

// Download new crates from crates.io
async fn download_crates(
    crates: &[Crate],
    dest_path: &Path,
    crates_io_base_url: &str,
    thread_num: u8,
) -> Result<(), Error> {
    let client = reqwest::Client::new();

    // Generate parallel async requests (via a stream of futures)
    let futures = futures::stream::iter(crates.iter().map(|c| {
        let client = &client;
        let download_url = generate_crate_url(c, crates_io_base_url);
        let local_crate_path = generate_crate_download_path(c, dest_path);

        info!("Downloading {} {} -> {}", c.name, c.version, download_url,);

        async move {
            match stream_to_file(client, &download_url, &local_crate_path).await {
                Ok(_) => (),
                Err(_) => error!("Error downloading {} {}", c.name, c.version),
            }
        }
    }))
    .buffer_unordered(usize::from(thread_num))
    .collect::<Vec<()>>();

    futures.await;

    Ok(())
}

// Traverse the index and determine if any new crates need to be downloaded
fn identify_new_crates(index_path: &Path, crate_path: &Path) -> Result<Vec<Crate>, Error> {
    let mut new_crates = Vec::new();

    let walker = WalkDir::new(index_path).into_iter();
    for entry in walker.filter_entry(|e| is_not_hidden(e) && is_not_config_file(e)) {
        let entry = entry?;
        if entry.path().is_file() {
            let file_path = &entry.path().display().to_string();
            let file = File::open(&file_path)?;
            let file = BufReader::new(file);

            for line in file.lines() {
                let curr_crate: Crate = match serde_json::from_str(&line?) {
                    Ok(c) => c,
                    Err(_) => {
                        // Something went wrong; try parsing next line in file
                        error!("Unable to parse line in index file {}", &file_path);
                        continue;
                    }
                };

                let local_crate_path = generate_crate_download_path(&curr_crate, crate_path);

                // Only add crate if it doesn't already exist in local repo
                if !Path::new(&local_crate_path).exists() {
                    info!(
                        "Identified new crate: {} {}",
                        curr_crate.name, curr_crate.version
                    );
                    new_crates.push(curr_crate)
                }
            }
        }
    }

    Ok(new_crates)
}

// Run the application
pub async fn run(opt: Opt) -> Result<(), Error> {
    // Get a list of new crates that need to be downloaded
    println!("\nDetermining new crates that need to be downloaded...");
    let new_crates = identify_new_crates(&opt.index_path, &opt.download_path)?;

    // Download new crates (if any)
    if !new_crates.is_empty() {
        println!(
            "\nEvaluation completed. Downloading new crates to {}...",
            &opt.download_path.display().to_string()
        );
        download_crates(
            &new_crates,
            &opt.download_path,
            &opt.crates_io_url,
            opt.threads,
        )
        .await?;
    } else {
        println!("\nNo new crates to download.");
    }

    // Done
    println!(
        "\nProcessing completed. Downloaded {} new crates.",
        &new_crates.len()
    );

    Ok(())
}
