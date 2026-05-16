use std::path::Path;

use anyhow::Result;
use brush_render::gaussian_splats::{self, SplatRenderMode, Splats};
use brush_serde::load_splat_from_ply;
use burn::backend::wgpu::WgpuDevice;
use burn::tensor::Device;
use tokio::io::BufReader;

use crate::message;

/// AI: Load a 3D Gaussian splat scene from a `.ply` file on disk.
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



pub fn render(request: message::ImageRequest, splats: Splats) {

    // let camera = brush_render::camera::Camera::new(
    //     position,           // Position 
    //     rotation,           // Rotation
    //     fov_x,              // FOV in x
    //     fov_y,              // FOV in y
    //     center_uv           // FIXME
    // )

    // gaussian_splats::render_splats(
    //     splats, 
    //     camera, 
    //     img_size, 
    //     background, 
    //     splat_scale, 
    //     texture_mode
    // );



    
    
}