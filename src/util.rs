use std::path::Path;

use anyhow::Result;
use brush_render::gaussian_splats::{self, SplatRenderMode, Splats};
// use brush_async::AsyncMap;
use brush_serde::load_splat_from_ply;
use burn::backend::wgpu::WgpuDevice;
use burn::tensor::Device;
use log::*;
use tokio::io::BufReader;

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



pub async fn render(
    request: &message::ImageRequest, 
    splats: Splats,
) {
    
    info!("Processing request #{}...", request.get_id());
    let position = request.get_camera_position();
    let rotation = request.get_camera_quaternion();
    // let mat_look = glam::Mat4::look_at_rh(position, glam::Vec3::new(0.0, 5.0, 0.0), glam::Vec3::new(0.0, 1.0, 0.0));

    // // let rotation = glam_quat([1.0, 0.0, 0.0, 0.0]);
    // let rotation = glam::Quat::from_mat4(&mat_look.inverse());
    // let center_uv = glam::Vec2::new(0.5, 0.5);
    // let img_size = glam::UVec2 { x: 800, y: 600 };
    // let background = glam::Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    // let mut camera = brush_render::camera::Camera::new(
    //     position,           // Position 
    //     rotation,           // Rotation
    //     90.0_f64.to_radians(),              // FOV in x
    //     60.0_f64.to_radians(),              // FOV in y
    //     center_uv           // FIXME
    // );

    // DEBUGGING


    // let means = splats.means();          // [N, 3]
    // let data = means.into_data();
    // let flat: Vec<f32> = data.to_vec().unwrap();
    // // let position = center + glam::Vec3::new(0.0, 0.0, extent);

    // let mut min = [f32::INFINITY; 3];
    // let mut max = [f32::NEG_INFINITY; 3];
    // for chunk in flat.chunks_exact(3) {
    //     for i in 0..3 {
    //         min[i] = min[i].min(chunk[i]);
    //         max[i] = max[i].max(chunk[i]);
    //     }
    // }
    // let center = [(min[0]+max[0])*0.5, (min[1]+max[1])*0.5, (min[2]+max[2])*0.5];
    // let extent = ((max[0]-min[0]).powi(2) + (max[1]-min[1]).powi(2) + (max[2]-min[2]).powi(2)).sqrt();
    // println!("bounds: min={:?} max={:?} center={:?} extent={:.2}", min, max, center, extent);


    // // let position = glam::Vec3::new(5.0, 5.0, 0.0);
    // let position = glam::Vec3::from_array(center) + glam::Vec3::new(0.0, 0.0, -extent);
    // let target   = glam::Vec3::from_array(center);
    // let forward  = (target - position).normalize();


    let camera = brush_render::camera::Camera::new(
        position,
        rotation,
        90.0_f64.to_radians(),
        60.0_f64.to_radians(),
        glam::Vec2::new(0.5, 0.5),
    );

    let img_size   = glam::UVec2::new(800*3, 600*3);
    let background = glam::Vec3::ZERO;


    let (image_tensor, _aux) = gaussian_splats::render_splats(
        splats, 
        &camera, 
        img_size, 
        background, 
        None, 
        gaussian_splats::TextureMode::Float,
    ).await;

    let tensor_raw = image_tensor.into_data();
    let floats: Vec<f32> = tensor_raw.to_vec().expect("expected f32 tensor");

    debug!("Size of output in floats: {}", floats.len());
    debug!("Expected for RGBA: {}", 800*600*4);

    let img_buffer: Vec<u8> = floats.iter()
        .map(|f| (f.clamp(0.0, 1.0) * 255.0) as u8)
        .collect();

    image::save_buffer("frame.png", &img_buffer, img_size[0], img_size[1], image::ColorType::Rgba8).unwrap();
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