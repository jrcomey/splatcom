# splatcom


## IPC Choices and Usage

So far I've decided on using the interprocess crate for IPC. I'm developing this on macOS, and locally this will yield a local socket, which behaves like a websocket but without using the network layer. Should enable sharing between processes on the same computer. Assuming that this may be transitioned to a network-based solution in future, this could be easily swapped out with a websocket if the need arises.

## AI Usage

* Dependency management for brush-render/brush-serde/burn package management. Brush subcrates depend on burn, but neither burn nor brush locks versions relative to each other. As of 15 MAY 2026, both projects have been updated multiple times today. Used Claude Code to lock down dependencies for each package relative to each other, which saved a large chunk of time without actually contributing to the design process.