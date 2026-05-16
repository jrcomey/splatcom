
type debug_field = bool; // Current debug item to indicate a field that should be replaced later.


struct ImageRequest {
    request_id: u64,                    //  FIXME Unique id (hash? integer? Integer means overflow problem. Check later.)
    timestamp: debug_field,             //  Timestamp from time lib
    camera_id: debug_field,             //  FIXME ID associated with camera? Unclear what this means. Maybe investigate transmitting camera lens data with JSON request.
    T_world_camera: debug_field,        // Camera transform. +X forward, +Z up. Quaternion configuration: [qw qx qy qz]
    intrinsics: debug_field,            // FIXME Pinhole camera intrinsics. Not sure what this refers to. FOV/other camera properties? Double check
}

impl ImageRequest {
    pub fn new() -> Self {

        todo!()
    }

    /// Takes in a recieved JSON string and attempts to parse it into an image request. Returns an error if the JSON is incomplete or contains bad information.
    pub fn new_from_json() -> Result<Self, Box<dyn std::error::Error>> {
        todo!()
    }



    
}


struct ImageResponse {
    request_id: u64,                    // Matching request ID from Image request
    timestamp: debug_field,             // 
    image_path: debug_field,
    width: debug_field,
    height: debug_field,
}