use std::path::Path;

use anyhow::Result;
use brush_render::gaussian_splats::{self, SplatRenderMode, Splats};
// use brush_async::AsyncMap;
use brush_serde::load_splat_from_ply;
use burn::backend::wgpu::WgpuDevice;
use burn::tensor::Device;
use log::*;
use tokio::io::BufReader;
use chrono;
use crate::message;

/// AI Function: Load a 3D Gaussian splat scene from a `.ply` file on disk.
///
/// `subsample_points` keeps approximately N points (passed straight through to
/// `brush_serde`); `None` keeps all of them.
pub async fn load_ply_file(filepath: impl AsRef<Path>, subsample_points: Option<u32>,
) -> Result<Splats> {
    let file = tokio::fs::File::open(filepath.as_ref()).await?;
    let reader = BufReader::new(file);

    let msg = load_splat_from_ply(reader, subsample_points).await?;
    let device: Device = WgpuDevice::default().into();
    let mode = msg.meta.render_mode.unwrap_or(SplatRenderMode::Default);

    Ok(msg.data.into_splats(&device, mode))
}



pub async fn render(request: &message::ImageRequest, splats: Splats) -> message::ImageResponse {

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

    let camera = brush_render::camera::Camera::new(
        position,
        rotation,
        90.0_f64.to_radians(),
        60.0_f64.to_radians(),
        glam::Vec2::new(0.5, 0.5),
    );

    // Hardcoded image size for now
    let img_size   = glam::UVec2::new(800*3, 600*3);
    let background = glam::Vec3::ZERO;


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

    // debug!("Size of output in floats: {}", floats.len());
    // debug!("Expected for RGBA: {}", 800*600*4);

    let img_buffer: Vec<u8> = floats.iter()
        .map(|f| (f.clamp(0.0, 1.0) * 255.0) as u8)
        .collect();

    
    // Same image
    let path = format!("output/frame_{}.png", request.get_id());
    image::save_buffer(&path, &img_buffer, img_size[0], img_size[1], image::ColorType::Rgba8).unwrap();
    info!("Saved request #{} to output/frame_{}", request.get_id(), request.get_id());

    let completion_time = chrono::Utc::now();

    message::ImageResponse::new(
        request.get_id(),
        &path,
        completion_time.to_string(),
        img_size[0] as u64,
        img_size[1] as u64,
        "png",
        (completion_time-request_time).num_microseconds().unwrap()
    )
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