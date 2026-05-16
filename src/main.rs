use anyhow::Result;
use serde_json;
use interprocess;
use glam;
use image;
mod util;
mod message;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use message as msg;

extern crate pretty_env_logger;
extern crate env_logger;
extern crate log;

use log::*;

async fn run_server(path: &str) -> Result<(), anyhow::Error> {

    // Broad architecture layout:
        // Load file
        // Spawn thread to start pulling in requests, which are placed in a buffer
        // As soon as file is loaded, start draining requests
            // In parallel? Have to figure out how that works on apple silicon with shared memory
        // Complete responses, dump in completed requests pile/heap/something
        // Drain responses in whatever means is actually necessary

    // Load File
    info!("Loading {}...", path);
    let splats = util::load_ply_file(&path, None).await?;
    info!("Loaded {} splats from {path}", splats.num_splats());


    // Spawn IPC reciever thread
    

    

    // Create ImageRequest queue
    let buffer: Arc<RwLock<VecDeque<msg::ImageRequest>>> = Arc::new(RwLock::new(VecDeque::new()));
    let sample_request = msg::ImageRequest::null();
    buffer.write().unwrap().push_back(sample_request.clone());
    buffer.write().unwrap().push_back(sample_request.clone());
    buffer.write().unwrap().push_back(sample_request.clone());
    buffer.write().unwrap().push_back(sample_request.clone());
    buffer.write().unwrap().push_back(sample_request.clone());

    info!("Beginning render loop...");


    // Main rendering loop

    while let Some(r) = buffer.read().unwrap().front() {
        let request = buffer.write().unwrap().pop_front().unwrap();
        util::render(request, splats.clone()).await;
    }

    
    
    info!("Done!");
    Ok(())
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

    run_server(&path).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::message::ImageRequest;

    use super::*;
    #[test]
    fn load_json() {
        let img: Result<ImageRequest, _> = serde_json::from_str(r#"{
        "request_id": 15,
        "timestamp" : "asdasas",
        "camera_id" : false,
        "T_world_camera" : [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
        "intrinsics" : false
    }"#);
        assert!(img.is_ok(), "Failed to produce image request from JSON!");
    }

    #[test]
    #[should_panic(expected="Failed to produce image request from JSON!")]
    fn wrong_json_size() {
        let img: Result<ImageRequest, _> = serde_json::from_str(r#"{
        "request_id": 15,
        "timestamp" : "asdasas",
        "camera_id" : false,
        "T_world_camera" : [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
        "intrinsics" : false
    }"#);
        assert!(img.is_ok(), "Failed to produce image request from JSON!");
    }
}