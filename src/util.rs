use crate::message;
use env_logger;
use log::*;
use brush_render::gaussian_splats::{self, Splats};


pub async fn render(request: &message::ImageRequest, splats: Splats) -> tokio::io::Result<message::ImageResponse> {

    info!("Processing request #{}...", request.get_id());

    // Measure server latency
    let render_start_time = chrono::Utc::now();
    let request_time: chrono::DateTime<chrono::Utc> = match chrono::DateTime::parse_from_rfc3339(request.get_timestamp()) {
        Ok(time) => time.into(),
        Err(_) => {
            warn!("Invalid time stamp recieved for request #{}! Latency metrics will not be valid for this request", request.get_id());
            chrono::Utc::now()
        }
    };


    // Construct Brush Camera from properties
    let position = request.get_camera_position();
    let rotation = request.get_camera_quaternion();
    let (mut fov_x, mut fov_y) = request.get_camera_fov();
    let (mut pinhole_x, mut pinhole_y) = request.get_pinhole_property();
    let (mut image_size_x, mut image_size_y) = request.get_image_size();
    

    // Invalid value handling
    if fov_x < 0.0 {
        warn!("Request recieved with invalid FOV value! Defualting to 90 deg");
        fov_x = 90.0;
    }
    if fov_y < 0.0 {
        warn!("Request recieved with invalid FOV value! Defualting to 90 deg");
        fov_y = 90.0;
    }


    if pinhole_x < 0.0 {
        warn!("Request recieved with invalid pinhole value! Defualting to 0.5");
        pinhole_x = 0.5;
    } else if pinhole_x > 1.0 {
        warn!("Request recieved with invalid pinhole value! Defualting to 0.5");
        pinhole_x = 0.5;
    }
    if pinhole_y < 0.0 {
        warn!("Request recieved with invalid pinhole value! Defualting to 0.5");
        pinhole_y = 0.5;
    } else if pinhole_y > 1.0 {
        warn!("Request recieved with invalid pinhole value! Defualting to 0.5");
        pinhole_y = 0.5;
    }

    let camera = brush_render::camera::Camera::new(
        position,
        rotation,
        (fov_x as f64).to_radians(),
        (fov_y as f64).to_radians(),
        glam::Vec2::new(pinhole_x, pinhole_y),
    );

    // Construct Image Size
    let img_size   = glam::UVec2::new(image_size_x, image_size_y);
    let background = glam::Vec3::new(1.0, 1.0, 1.0);


    // From brush code
    let (image_tensor, _aux) = gaussian_splats::render_splats(
        splats,
        &camera,
        img_size,
        background,
        None,
        gaussian_splats::TextureMode::Float,
    ).await;


    // Process tensor into image
    let tensor_raw = image_tensor.into_data();
    let floats: Vec<f32> = tensor_raw.to_vec().expect("expected f32 tensor");
    let img_buffer: Vec<u8> = floats.iter()
        .map(|f| (f.clamp(0.0, 1.0) * 255.0) as u8)
        .collect();

    
    // Same image
    std::fs::create_dir_all("output")?;
    let path = format!("output/frame_{}.png", request.get_id());
    image::save_buffer(&path, &img_buffer, img_size[0], img_size[1], image::ColorType::Rgba8)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    info!("Saved request #{} to output/frame_{}", request.get_id(), request.get_id());

    let completion_time = chrono::Utc::now();

    let output = message::ImageResponse::new(
        request.get_id(),
        &completion_time.to_string(),
        &path,
        img_size[0] as u64,
        img_size[1] as u64,
        "png",
        (completion_time-request_time).num_microseconds().unwrap());

    Ok(output)
}

/// Helper function to construct quaternions from different convention
pub fn glam_quat(init_quat: [f32; 4]) -> glam::Quat {
    
    glam::Quat::from_array([
        init_quat[1], 
        init_quat[2], 
        init_quat[3], 
        init_quat[0], 
    ])
}