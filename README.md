# splatcom

Single-client gaussian splat rendering process designed to provide simulated camera data for simulation use.

## Client Setup

Create a python virtual environment using the following:

```bash
python -m venv .env
source .env/bin/activate
```

Install the test client package with the following:

```bash
pip install -e ./basic_client
```

## Usage

Simply call the splatcom executable with the relative filepath of the .ply file you need to render. E.g.

```bash
cargo r -- data/construction_site.ply
```

The server is also launchable through Docker using the following commands:

```bash
docker build -t splatcom . 
docker run --rm -v "$(pwd)/data:/data:ro" -p 127.0.0.1:8080:8080 splatcom /data/construction_site.ply
```

Assuming your model is located in the `data/` directory. 

NOTE: The containerization is untested. This server was developed on macOS, and there is no compatibility layer for calls to the M4 GPU from a Linux image. It should work on any other hardware, but *is* untested. 

## Dependencies

Below is a list of dependencies and the reasons they are included:

* `brush-render` | Rendering crate from `brush`
* `brush-render` | IO crate from `brush`
* `serde-json` | JSON parser
* `interprocess` | Flexible handling for IPC, explained below
* `log/env_logger/pretty_env_logger` | crate for handling logging statements
* `glam` | Vector graphics library used by `brush`
* `anyhow` | multipurpose error handling crate
* `tokio` | Asynchronous rust, used for both async threads and networking

## IPC Choices and Usage

~~So far I've decided on using the interprocess crate for IPC. I'm developing this on macOS, and locally this will yield a local socket, which behaves like a websocket but without using the network layer. Should enable sharing between processes on the same computer. Assuming that this may be transitioned to a network-based solution in future, this could be easily swapped out with a websocket if the need arises.~~

`interprocess` has been swapped out for Tokio. I misread the original prompt about shared memory and assumed requests had to be sent through shared memory, not just saved images. Server now communicates over TCP on `127.0.0.1`, and is configured to work whether on Docker or built locally.

### Message Passing

I have chosen to utilize JSON for the time being. Protobuf is a better choice for performance, but I'm currently more familiar with JSON and the `serde-json` library, and will use that at the start of the development cycle. Switching between JSON and Protobuf would be relatively simple in future, but would cause breaking changes (assuming we're not going to have both message passing systems be backwards compatible)

### General Program Structure

`splatcom` uses two main threads: one handles network I/O, and the other is the primary rendering loop.

The network thread starts a TCP listener and awaits incoming image request packets. The packets are deserialized using `serde-json`, and pushed to an image request buffer as they are received. Each incoming packet uses its own `tokio` thread, which prevents blockage while creating minimal overhead. Each job is sent with a `tokio` oneshot reply channel (essentially a consumable `mpsc` thread transmission channel) which returns the `ImageResponse` with render metrics and the delivered file location. The resultant packet is then transmitted back on the same listener thread.

The primary render loop uses `brush` as a rendering backend. Splats are loaded with `brush`'s gaussain splat backend, and stored in VRAM. Camera position, rotation, and other characteristics such as FOV are transmitted with the image request, and used in the primary render call. The resultant tensor is processed from floats into RGBA, saved to file, and the metrics are returned to the I/O thread.

## AI Usage

* Dependency management for `brush-render`/`brush-serde`/`burn` package management. Brush subcrates depend on burn, but neither burn nor brush locks versions relative to each other. As of 15 MAY 2026, both projects have been updated multiple times today. Used Claude Code to lock down dependencies for each package relative to each other, which saved a large chunk of time without actually contributing to the design process.
* Documentation for `brush` backend, which contains little to no comments or documentation (e.g. camera FOV is in radians, not degrees)
* Some minor debugging code not used in the final build (e.g. generate a list of points and save off images at each point) as a time saving measure
* A ctrl-c shutdown feature which stopped the socket from being blocked when the program was closed
* Sender file functions to generate a sphere of points and send them over the TCP port. Something I can do but more of a time saving thing.
* Docker tutorial