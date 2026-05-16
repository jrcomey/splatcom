use anyhow::Result;
use serde_json;
use interprocess;
mod util;
mod message;

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

    let args: Vec<String> = std::env::args().collect();
    run_server(&args[0]);


    // Some AI loading code that I'm not using but it generated anyway
    // let mut args = std::env::args().skip(1);

    // let Some(path) = args.next() else {
    //     eprintln!("usage: splatcom <path-to-ply>");
    //     return Ok(());
    // };

    // let splats = util::load_ply_file(&path, None).await?;
    // println!("Loaded {} splats from {path}", splats.num_splats());

    Ok(())
}
