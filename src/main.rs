#![deny(warnings)]

use crate_downloader::Opt;
use env_logger;
use structopt::StructOpt;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Make sure args are provided
    let opt = Opt::from_args();

    // Run the app
    if let Err(e) = crate_downloader::run(opt).await {
        println!("Application error: {}", e);
        std::process::exit(1);
    }
}
