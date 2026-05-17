use anyhow::Result;
use interprocess::local_socket::traits::ListenerExt;
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};
use serde_json;
use interprocess;
use glam;
use image;
mod util;
mod message;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
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

    // Incoming buffer setup
    let inbox: Arc<RwLock<VecDeque<msg::RenderJob>>> = Arc::new(RwLock::new(VecDeque::new()));


    // Spawn IPC TX/RX thread
    let inbox_ipc = inbox.clone(); // Free clone of ref
    tokio::spawn(async move {
        let namespace = "splatcom.sock".to_ns_name::<GenericNamespaced>().unwrap();
        let listener = ListenerOptions::new().name(namespace).create_sync().unwrap();
        for connection in listener.incoming().filter_map(|conn| conn.ok()){
            let mut connection = BufReader::new(connection);
            let mut incoming_json = String::new();
            connection.read_line(&mut incoming_json).unwrap();
            if let Ok(request) = serde_json::from_str::<msg::ImageRequest>() {
                
            }
            let (job_tx, job_rx) = std::sync::mpsc::channel();
            
        }
    });


    

    // Create ImageRequest queue
    
    // let sample_request = msg::ImageRequest::null();
    // inbox.write().unwrap().push_back(sample_request.clone());
    // inbox.write().unwrap().push_back(sample_request.clone());
    // inbox.write().unwrap().push_back(sample_request.clone());
    // inbox.write().unwrap().push_back(sample_request.clone());
    // inbox.write().unwrap().push_back(sample_request.clone());

    info!("Beginning render loop...");


    // Main rendering loop

    while let Some(r) = inbox.read().unwrap().front() {
        let job = inbox.write().unwrap().pop_front().unwrap();
        util::render(job.get_request(), splats.clone()).await;
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