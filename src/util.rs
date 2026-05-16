use std::path::Path;

use anyhow::Result;
use brush_render::gaussian_splats::{self, SplatRenderMode, Splats};
use brush_serde::load_splat_from_ply;
use burn::backend::wgpu::WgpuDevice;
use burn::tensor::Device;
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



pub async fn render(request: message::ImageRequest, splats: Splats) {
    
    let position = glam::Vec3 { x: 10.0, y: 10.0, z: 10.0 };
    let rotation = glam_quat([1.0, 0.0, 0.0, 0.0]);
    let center_uv = glam::Vec2::new(100.0, 100.0);
    let img_size = glam::UVec2 { x: 100, y: 100 };
    let background = glam::Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let camera = brush_render::camera::Camera::new(
        position,           // Position 
        rotation,           // Rotation
        100.0,              // FOV in x
        100.0,              // FOV in y
        center_uv           // FIXME
    );

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

    let proc_floats: Vec<u8> = floats.iter()
        .map(|f| (f.clamp(0.0, 1.0) * 255.0) as u8)
        .collect();

    

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