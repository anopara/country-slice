pub struct GlTextureRGBAf32 {
    pub id: u32,
    pub dims: (i32, i32),
}

impl GlTextureRGBAf32 {
    pub fn new(dims: (i32, i32), raw_f32_pixels: Option<&Vec<f32>>) -> Self {
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);

            // Create a pointer to pixel data, or, if no pixels are provided, make a null ptr
            let pixels = raw_f32_pixels
                .map(|p| &p[0] as *const f32 as *const std::ffi::c_void)
                .unwrap_or(std::ptr::null());
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA32F as i32,
                dims.0,
                dims.1,
                0,
                gl::RGBA,
                gl::FLOAT,
                pixels,
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        Self { id: texture, dims }
    }

    pub fn update(&mut self, raw_f32_pixels: &[f32]) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);

            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                self.dims.0,
                self.dims.1,
                gl::RGBA,
                gl::FLOAT,
                &raw_f32_pixels[0] as *const f32 as *const std::ffi::c_void,
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            let raw_pixels: Vec<[f32; 4]> =
                vec![[0.0, 0.0, 0.0, 1.0]; (self.dims.0 * self.dims.1) as usize];

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA32F as i32,
                self.dims.0,
                self.dims.1,
                0,
                gl::RGBA,
                gl::FLOAT,
                &raw_pixels[0] as *const f32 as *const std::ffi::c_void,
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}
