/// Contains AI generated code. May be unsafe!!

use std::path::Path;
use anyhow::Result;
use tokio::io::BufReader;
use burn::backend::wgpu::WgpuDevice;
use burn::tensor::Device;
use brush_serde::load_splat_from_ply;
use brush_render::gaussian_splats::{SplatRenderMode, Splats};



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