
type debug_field = bool;


struct ImageRequest {
    request_id: u64,
    timestamp: debug_field,
    camera_id: debug_field,
    T_world_camera: debug_field,
    intrinsics: debug_field,
}


struct ImageResponse {
    request_id: u64,
    timestamp: debug_field,
    image_path: debug_field,
    width: debug_field,
    height: debug_field,
}