use std::time::Instant;

use crate::util::glam_quat;


type debug_field = bool; // Current debug item to indicate a field that should be replaced later.



pub struct ImageRequest {
    request_id: u64,                    //  FIXME Unique id (hash? integer? Integer means overflow problem. Check later.)
    timestamp: Instant,             //  Timestamp from time lib
    camera_id: debug_field,             //  FIXME ID associated with camera? Unclear what this means. Maybe investigate transmitting camera lens data with JSON request.
    T_world_camera: [f32; 7],        // Camera transform. +X forward, +Z up. Quaternion configuration: [qw qx qy qz]
    intrinsics: debug_field,            // FIXME Pinhole camera intrinsics. Not sure what this refers to. FOV/other camera properties? Double check
}

impl ImageRequest {

    /// Basic Constructor for ImageRequest
    pub fn new() -> Self {
        ImageRequest { ..Default::default()}
    }

    /// Takes in a recieved JSON string and attempts to parse it into an image request. Returns an error if the JSON is incomplete or contains bad information.
    pub fn new_from_json() -> Result<Self, Box<dyn std::error::Error>> {
        todo!()
    }

    pub fn camera_position(&self) -> glam::Vec3 {
        glam::Vec3 { 
            x: self.T_world_camera[0], 
            y: self.T_world_camera[1], 
            z: self.T_world_camera[2] }
    }

    pub fn camera_quaternion(&self) -> glam::Quat {
        glam_quat([
            self.T_world_camera[3],         // quat W
            self.T_world_camera[4],         // quat X
            self.T_world_camera[5],         // quat Y
            self.T_world_camera[6]          // quat Z
        ])
    }
}


impl Default for ImageRequest {
    fn default() -> Self {
        ImageRequest {
            request_id: 0, 
            timestamp: Instant::now(), 
            camera_id: false, 
            T_world_camera: [0.0; 7], 
            intrinsics: false 
        }
    }
}

struct ImageResponse {
    request_id: u64,                    // Matching request ID from Image request
    timestamp: Instant,                 // Server completion time
    image_path: String,            // Resultant Image Path
    width: u64,                         // Image width in pixels
    height: u64,                        // Image height in pixels
    dtype: debug_field,                  // Image type
    stride: debug_field,                // FIXME: No idea. 
    render_latency_us: u64,             // Server render latency in us
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
