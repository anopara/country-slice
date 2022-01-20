use super::compute_textures::ComputeTexture;

// path mask is a texture/plane centered on 0.0 with bounds from -10 to 10
pub const PATH_MASK_WS_DIMS: [f32; 2] = [20.0, 20.0];

pub struct ComputePathMask(pub ComputeTexture);
pub struct ComputePathBlur(pub ComputeTexture);
