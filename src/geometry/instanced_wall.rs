use glam::Mat4;

use crate::render::ssbo::GLShaderStorageBuffer;

use super::wall_constructor::Brick;

const SSBO_BUFFER_SIZE: usize = 10000;
const SSBO_BINDING_POINT: u32 = 2;

#[repr(C)]
pub struct InstancedWall {
    pub wall_length: f32,
    pub instance_buffer: GLShaderStorageBuffer<BrickTransformSSBO>,
}

impl InstancedWall {
    fn instanced_wall_data(bricks: Vec<Brick>) -> Vec<BrickTransformSSBO> {
        bricks
            .iter()
            .map(|b| {
                let min = b.pivot_uv - b.bounds_uv / 2.0;
                let max = b.pivot_uv + b.bounds_uv / 2.0;

                BrickTransformSSBO {
                    transform: b.transform.compute_matrix(),
                    curve_uv_bbx_minmax: [min.x, min.y, max.x, max.y],
                }
            })
            .collect()
    }

    pub fn from(curve_length: f32, bricks: Vec<Brick>) -> Self {
        Self {
            wall_length: curve_length,
            instance_buffer: GLShaderStorageBuffer::<BrickTransformSSBO>::new(
                &Self::instanced_wall_data(bricks),
                SSBO_BUFFER_SIZE,
                SSBO_BINDING_POINT,
            ),
        }
    }

    pub fn update(&mut self, curve_length: f32, bricks: Vec<Brick>) {
        self.wall_length = curve_length;
        self.instance_buffer
            .update(&Self::instanced_wall_data(bricks));
    }

    pub fn free_memory(&mut self) {
        //TODO:
        unimplemented!()
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct BrickTransformSSBO {
    transform: Mat4,
    curve_uv_bbx_minmax: [f32; 4],
}
