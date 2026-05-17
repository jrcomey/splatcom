use anyhow::Result;
// use interprocess::local_socket::traits::ListenerExt;
use interprocess::local_socket::traits::tokio::Listener;
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};
use serde_json;
use interprocess;
use glam;
use image;
mod util;
mod message;
use std::collections::VecDeque;
// use std::io::{BufRead, BufReader};
// use tokio::io::BufReader;
use tokio::io::{AsyncBufReadExt, BufReader};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use message as msg;

extern crate pretty_env_logger;
extern crate env_logger;
extern crate log;

use log::*;

use crate::message::ImageResponse;

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
    let shutdown_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));


    // Spawn IPC TX/RX thread
    info!("Spawning IPC thread...");
    let inbox_ipc = inbox.clone(); // Free clone of ref
    let shutdown_flag_ipc = shutdown_flag.clone();
    tokio::spawn(async move {
        // Spawn socket for a single client
        let namespace = "splatcom.sock".to_ns_name::<GenericNamespaced>().unwrap();
        let listener = ListenerOptions::new().name(namespace).create_tokio().unwrap();

        loop {
            let connection = match listener.accept().await {
                Ok(conn) => conn,
                _ => continue,
            };

            if shutdown_flag_ipc.load(Ordering::Relaxed) {
                break;
            }

            let inbox_ipc_clone = inbox_ipc.clone();
            tokio::spawn(
                async move {
                    let mut connection = BufReader::new(connection);
                    let mut incoming_json = String::new();
                    connection.read_line(&mut incoming_json).await;
                    if let Ok(request) = serde_json::from_str::<msg::ImageRequest>(&incoming_json) {
                        // Build job, send to buffer, reply to connection with response
                        info!("Recieved request {}!", request.get_id());
                        let (job_tx, mut job_rx) = tokio::sync::oneshot::channel::<msg::ImageResponse>();
                        inbox_ipc_clone.write().unwrap().push_back(msg::RenderJob::new(request, job_tx));
                        // let job_done: ImageResponse = job_rx.blocking_recv().unwrap();
                        // TODO Reply
                    }
                }
            );
        }

        // Create a new tokio thread per incoming request. If it's valid, build the job, send to the buffer, and await the response. When you recieve the response, reply with image
        // for connection in listener.incoming().filter_map(|conn| conn.ok()){
        //     if shutdown_flag_ipc.load(Ordering::Relaxed) {
        //         info!("Terminating IPC loop");
        //         break;
        //     }
        //     let inbox_ipc_clone = inbox_ipc.clone();
        //     tokio::spawn(
        //         async move {
        //             let mut connection = BufReader::new(connection);
        //             let mut incoming_json = String::new();
        //             connection.read_line(&mut incoming_json).unwrap();
        //             if let Ok(request) = serde_json::from_str::<msg::ImageRequest>(&incoming_json) {
        //                 // Build job, send to buffer, reply to connection with response
        //                 info!("Recieved request {}!", request.get_id());
        //                 let (job_tx, job_rx) = tokio::sync::oneshot::channel::<msg::ImageResponse>();
        //                 inbox_ipc_clone.write().unwrap().push_back(msg::RenderJob::new(request, job_tx));
        //                 let job_done: ImageResponse = job_rx.blocking_recv().unwrap();
        //                 // TODO Reply
        //             }
        //         }
                
        //     );
        //     // if *shutdown_flag.read().unwrap() {
        //     //     drop(listener);
        //     // }
        // }
    });

    info!("Beginning render loop...");

    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    // Main rendering loop
    loop {
        tokio::select! {
            biased;
            _ = &mut ctrl_c => {
                info!("Ctrl-C received, shutting down");
                shutdown_flag.store(true, Ordering::Relaxed);
                break;
            }
            _ = async {
                let job = inbox.write().unwrap().pop_front();
                match job {
                    Some(job) => {
                        let (request, reply) = job.into_parts();
                        let response = util::render(&request, splats.clone()).await;
                        let _ = reply.send(response);
                    }
                    _ => tokio::task::yield_now().await,
                }
            } => {}
        }
    }

    info!("Done!");
    Ok(())
}

#[tokio::main]
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