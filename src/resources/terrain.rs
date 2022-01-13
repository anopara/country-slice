use bracket_noise::prelude::FastNoise;

pub struct TerrainData {
    perlin: bracket_noise::prelude::FastNoise,
    pub amp: f32,
    pub offset: glam::Vec2,
    pub min_y: f32,
    pub max_y: f32,
    pub texture: u32,
    pub texture_dims: (i32, i32),
}

impl TerrainData {
    pub fn recalculate_texture(&mut self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            let (raw_pixels, min, max) =
                Self::raw_pixels_f32(&self.perlin, self.texture_dims, self.offset, self.amp);

            self.min_y = min;
            self.max_y = max;

            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                self.texture_dims.0,
                self.texture_dims.1,
                gl::RGBA,
                gl::FLOAT,
                &raw_pixels[0] as *const f32 as *const std::ffi::c_void,
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn height_at(&self, x: f32, y: f32) -> f32 {
        self.perlin.get_noise(self.offset.x + x, self.offset.y + y) * self.amp
    }

    #[allow(dead_code)]
    pub fn raw_empty_f32(texture_dims: (i32, i32)) -> (Vec<f32>, f32, f32) {
        (
            vec![0.0; (texture_dims.1 * texture_dims.0 * 4) as usize],
            0.0,
            0.0,
        )
    }

    #[allow(dead_code)]
    pub fn raw_pixels_f32(
        noise: &FastNoise,
        texture_dims: (i32, i32),
        offset: glam::Vec2,
        amp: f32,
    ) -> (Vec<f32>, f32, f32) {
        let size = (20.0, 20.0);
        let mut raw_pixels = Vec::new();

        let mut min_value = 0.0;
        let mut max_value = 0.0;

        for y in 0..texture_dims.1 {
            let p_y = (y as f32 / texture_dims.1 as f32) * size.1 - size.1 / 2.0;

            for x in 0..texture_dims.0 {
                let p_x = (x as f32 / texture_dims.0 as f32) * size.0 - size.0 / 2.0;

                let n = noise.get_noise(p_x + offset.x, p_y + offset.y) * amp;
                raw_pixels.extend([n, n, n, 1.0]);

                if n < min_value {
                    min_value = n;
                }

                if n > max_value {
                    max_value = n;
                }
            }
        }

        (raw_pixels, min_value, max_value)
    }

    pub fn new() -> Self {
        let mut noise = bracket_noise::prelude::FastNoise::seeded(45);
        noise.set_noise_type(bracket_noise::prelude::NoiseType::PerlinFractal);
        noise.set_fractal_type(bracket_noise::prelude::FractalType::FBM);
        noise.set_fractal_octaves(3);
        noise.set_fractal_gain(1.0);
        noise.set_fractal_lacunarity(3.0);
        noise.set_frequency(0.05);
        noise.set_seed(0);

        let amp = 1.3;
        let offset = glam::Vec2::ZERO;
        let texture_dims = (512, 512);

        // generate texture
        let min_y;
        let max_y;
        let texture = unsafe {
            let (raw_pixels, min, max) = Self::raw_empty_f32(texture_dims); //Self::raw_pixels_f32(&noise, texture_dims, offset, amp);

            min_y = min;
            max_y = max;
            // Create texture
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA32F as i32,
                texture_dims.0,
                texture_dims.1,
                0,
                gl::RGBA,
                gl::FLOAT,
                &raw_pixels[0] as *const f32 as *const std::ffi::c_void,
            );
            texture
        };

        Self {
            perlin: noise,
            min_y,
            max_y,
            texture,
            amp,
            offset,
            texture_dims,
        }
    }
}
