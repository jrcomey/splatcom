use anyhow::Result;
use serde_json;
use std::collections::VecDeque;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    {Arc, RwLock}
};
use pretty_env_logger;
use env_logger;
use log::*;

mod util;
mod message;
mod clanker;
use message as msg;


/// Runs the primary render loop. Recieves image requests aysnchronously, generates the images by pulling requests from the queue, and 
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
    let splats = clanker::load_ply_file(&path, None).await?;
    info!("Loaded {} splats from {path}", splats.num_splats());

    // Incoming buffer setup
    let inbox: Arc<RwLock<VecDeque<msg::RenderJob>>> = Arc::new(RwLock::new(VecDeque::new()));
    let shutdown_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));


    // Spawn IPC TX/RX thread
    info!("Spawning IPC thread...");
    let inbox_ipc = inbox.clone(); // Free clone of ref
    let shutdown_flag_ipc = shutdown_flag.clone();
    tokio::spawn(async move {
        // Spawn socket for a single client, check if there's an env set from docker or no
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
        let listener = TcpListener::bind(&bind_addr).await.unwrap();
        info!("listening on {bind_addr}");

        loop {
            let (stream, socket) = match listener.accept().await {
                Ok(conn) => (conn.0, conn.1),
                _ => continue,
            };

            if shutdown_flag_ipc.load(Ordering::Relaxed) {
                break;
            }

            let inbox_ipc_clone = inbox_ipc.clone();
            // let (read_half, mut write_half) = connection.into_split();
            tokio::spawn(
                async move {
                    let (read_half, mut write_half) = stream.into_split();
                    let mut read_half = BufReader::new(read_half);
                    let mut incoming_json = String::new();
                    read_half.read_line(&mut incoming_json).await.unwrap();
                    if let Ok(request) = serde_json::from_str::<msg::ImageRequest>(&incoming_json) {
                        // Build job, send to buffer, reply to connection with response
                        info!("Recieved request {}!", request.get_id());
                        let (job_tx, mut job_rx) = tokio::sync::oneshot::channel::<msg::ImageResponse>();
                        // Send job to buffer, wait for reply
                        inbox_ipc_clone.write().unwrap().push_back(msg::RenderJob::new(request, job_tx));
                        let job_done: msg::ImageResponse = job_rx.await.unwrap();

                        // Send reply JSON to client
                        let str_reply = serde_json::to_string(&job_done).unwrap();
                        write_half.write_all(str_reply.as_bytes()).await.unwrap();
                    }
                }
            );
        }
    });

    info!("Beginning render loop...");

    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    // Main rendering loop
    loop {
        // Shutdown server, otherwise continue checking buffer
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
                        let _ = reply.send(response.unwrap());
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