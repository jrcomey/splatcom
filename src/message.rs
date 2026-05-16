use std::str::FromStr;
use std::time::Instant;
use std::collections::VecDeque;
use crate::util::glam_quat;
use serde::Deserialize;
use serde_json;
use log::*;


type debug_field = bool; // Current debug item to indicate a field that should be replaced later.


// /// Buffer of Image Requests
// pub struct RequestBuffer {
//     buffer: VecDeque<ImageRequest>,
// }

#[derive(Clone, Deserialize)]
pub struct ImageRequest {
    request_id: u64,                        //  FIXME Unique id (hash? integer? Integer means overflow problem. Check later.)
    timestamp: String,                     //  Timestamp from time lib
    camera_id: debug_field,                 //  FIXME ID associated with camera? Unclear what this means. Maybe investigate transmitting camera lens data with JSON request.
    T_world_camera: [f32; 7],               // Camera transform. +X forward, +Z up. Quaternion configuration: [qw qx qy qz]
    intrinsics: debug_field,                // FIXME Pinhole camera intrinsics. Not sure what this refers to. FOV/other camera properties? Double check
}

impl ImageRequest {

    /// Basic Constructor for ImageRequest, to be filled with actual IO later
    pub fn new() -> Self {
        ImageRequest { ..Default::default()}
    }

    pub fn null() -> Self {
        ImageRequest { ..Default::default()}
    }

    // /// Takes in a recieved JSON string and attempts to parse it into an image request. Returns an error if the JSON is incomplete or contains bad information.
    // pub fn new_from_json(json_str: String) -> Result<Self, Box<dyn std::error::Error>> {
    //     let json_parsed = serde_json::Value::from_str(&json_str)?;

    //     // Pull request ID
    //     let id = if let Some(id) = json_parsed.get("id") {
    //         id
    //     } else {
    //         return Err(std::fmt::Error);
    //     };

    //     // Pull client send time

    //     // Pull camera ID (nothing for now)

    //     // Pull 

    //     todo!()
    // }

    pub fn get_camera_position(&self) -> glam::Vec3 {
        glam::Vec3 { 
            x: self.T_world_camera[0], 
            y: self.T_world_camera[1], 
            z: self.T_world_camera[2] }
    }

    pub fn get_camera_quaternion(&self) -> glam::Quat {

        let mut quat = glam_quat([
            self.T_world_camera[3],         // quat W
            self.T_world_camera[4],         // quat X
            self.T_world_camera[5],         // quat Y
            self.T_world_camera[6]          // quat Z
        ]);

        if quat == glam::Quat::from_array([0.0, 0.0, 0.0, 0.0]) {
            warn!("All zero quaternion detected! Using unit quaternion instead.");
            quat = glam_quat([1.0, 0.0, 0.0, 0.0]);
        }
        quat.normalize() 
    }

    pub fn get_id(&self) -> u64 {
        self.request_id
    }
}


impl Default for ImageRequest {
    fn default() -> Self {
        ImageRequest {
            request_id: 0, 
            timestamp: "".to_string(), 
            camera_id: false, 
            T_world_camera: [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0], 
            intrinsics: false 
        }
    }
}

struct ImageResponse {
    request_id: u64,                        // Matching request ID from Image request
    timestamp: Instant,                     // Server completion time
    image_path: String,                     // Resultant Image Path
    width: u64,                             // Image width in pixels
    height: u64,                            // Image height in pixels
    dtype: debug_field,                     // Image type
    stride: debug_field,                    // FIXME: No idea. 
    render_latency_us: u64,                 // Server render latency in us
}

impl ImageResponse {
    // Basic constructor
    pub fn new(request_id: u64, image_path: String, width: u64, height: u64) -> Self {
        ImageResponse { request_id, timestamp: Instant::now(), image_path, width, height,..Default::default() }
    }
}


impl Default for ImageResponse {
    fn default() -> Self {
        ImageResponse {
            request_id: 0, 
            timestamp: Instant::now(), 
            image_path: "".to_string(), 
            width: 0, 
            height: 0,
            dtype: false,
            stride: false,
            render_latency_us: 0 }
    }
}
