#[allow(non_snake_case)]
use crate::util::glam_quat;
use serde::{Deserialize, Serialize};
use log::*;
use tokio::sync::oneshot;



type debug_field = bool; // Current debug item to indicate a field that should be replaced later.


pub struct RenderJob {
    request: ImageRequest,
    reply_channel: oneshot::Sender<ImageResponse>,
}

impl RenderJob {
    pub fn new(request: ImageRequest, reply_channel: oneshot::Sender<ImageResponse>) -> Self {
        RenderJob { request, reply_channel }
    }

    pub fn get_request(&self) -> &ImageRequest {
        &self.request
    }

    pub fn into_parts(self) -> (ImageRequest, oneshot::Sender<ImageResponse>) {
        (self.request, self.reply_channel)
    }
}

/// Image request struct, recieved over network as JSON
#[derive(Clone, Deserialize)]
pub struct ImageRequest {
    request_id: u64,                        //  Unique id (hash? integer? Integer means overflow problem. Check later.)
    timestamp: String,                      //  Timestamp from chrono, in UTC format (RFC3339 format)
    camera_id: debug_field,                 //  Camera ID, if needed on the client end
    T_world_camera: [f32; 7],               //  Camera transform. +X forward, +Z up. Quaternion configuration: [qw qx qy qz]
    intrinsics: debug_field,                //  FIXME Pinhole camera intrinsics. Not sure what this refers to. FOV/other camera properties? Double check
}

impl ImageRequest {


    /// Returns camera position
    pub fn get_camera_position(&self) -> glam::Vec3 {
        glam::Vec3 { 
            x: self.T_world_camera[0], 
            y: self.T_world_camera[1], 
            z: self.T_world_camera[2] }
    }

    /// Returns camera quaternion. Handles all zero edge case
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

    /// Returns id of image request
    pub fn get_id(&self) -> u64 {
        self.request_id
    }

    /// Returns timestamp (in string form)
    pub fn get_timestamp(&self) -> &str {
        &self.timestamp
    }
}


impl Default for ImageRequest {
    /// Defaults!
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

/// Image response struct, sent over network as JSON
#[derive(Serialize)]
pub struct ImageResponse {
    request_id: u64,                        // Matching request ID from Image request
    timestamp: String,                      // Server completion time
    image_path: String,                     // Resultant Image Path
    width: u64,                             // Image width in pixels
    height: u64,                            // Image height in pixels
    dtype: String,                          // Image type
    stride: debug_field,                    // FIXME: No idea. 
    render_latency_us: i64,                 // Server render latency in us
}

impl ImageResponse {
    // Basic constructor
    pub fn new(request_id: u64, time: &str, image_path: String, width: u64, height: u64, dtype: &str, latency_time_us: i64) -> Self {
        ImageResponse { request_id, timestamp: time.to_string(), image_path, width, height, dtype: dtype.to_string(), stride: false, render_latency_us: latency_time_us}
    }
}


impl Default for ImageResponse {
    fn default() -> Self {
        ImageResponse {
            request_id: 0, 
            timestamp: "".to_string(), 
            image_path: "".to_string(), 
            width: 0, 
            height: 0,
            dtype: "png".to_string(),
            stride: false,
            render_latency_us: 0 }
    }
}
