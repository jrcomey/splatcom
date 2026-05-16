use anyhow::Result;
use serde_json;
use interprocess;
use glam;
mod util;
mod message;

extern crate pretty_env_logger;
extern crate env_logger;
extern crate log;

use log::*;

fn run_server(filepath: &str) {

    // Broad architecture layout:
        // Load file
        // Spawn thread to start pulling in requests, which are placed in a buffer
        // As soon as file is loaded, start draining requests
            // In parallel? Have to figure out how that works on apple silicon with shared memory
        // Complete responses, dump in completed requests pile/heap/something
        // Drain responses in whatever means is actually necessary

    util::load_ply_file(filepath, None);
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    unsafe {
        std::env::set_var("RUST_LOG", "splatcom=trace,warn cargo run"); // Initialize logger
    }
    pretty_env_logger::init();
    let mut args = std::env::args().skip(1);

    let Some(path) = args.next() else {
        error!("usage: splatcom <path-to-ply>");
        return Ok(());
    };

    info!("Loading {}...", path);
    let splats = util::load_ply_file(&path, None).await?;
    info!("Loaded {} splats from {path}", splats.num_splats());

    Ok(())
}
