# splatcom

Single-client gaussian splat rendering process designed to provide simulated camera data for simulation use.



## IPC Choices and Usage

So far I've decided on using the interprocess crate for IPC. I'm developing this on macOS, and locally this will yield a local socket, which behaves like a websocket but without using the network layer. Should enable sharing between processes on the same computer. Assuming that this may be transitioned to a network-based solution in future, this could be easily swapped out with a websocket if the need arises.

### Message Passing

I have chosed to utilize JSON for the time being. Protobuf is a better choice for performance, but I'm currently more familiar with JSON and the `serde-json` library, and will use that at the start of the development cycle. Switching between JSON and Protobuf would be relatively simple in future, but would cause breaking changes (assuming we're not going to have both message passing systems be backwards compatible)

## AI Usage

* Dependency management for brush-render/brush-serde/burn package management. Brush subcrates depend on burn, but neither burn nor brush locks versions relative to each other. As of 15 MAY 2026, both projects have been updated multiple times today. Used Claude Code to lock down dependencies for each package relative to each other, which saved a large chunk of time without actually contributing to the design process.