struct TestMask {
    _ndc_quad_mesh: Mesh,
    ndc_quad_vao: VAO,
    shader: ShaderProgram,
    framebuffer: u32,
    texture_buffer: u32,
    texture_dims: (i32, i32),
}

fn test_mask_w_framebuffers(texture_dims: (i32, i32)) -> TestMask {
    // Tets mask
    let shader = ShaderProgram::new("shaders/draw_mask.vert", "shaders/draw_mask.frag").unwrap();

    let mut ndc_quad_mesh = Mesh::new();
    ndc_quad_mesh.set_attribute(
        "NDC_Position",
        vec![[-1.0, 1.0], [-1.0, -1.0], [1.0, -1.0], [1.0, 1.0]],
    );
    ndc_quad_mesh.set_indices(vec![0, 1, 2, 0, 2, 3]);

    // create framebuffer
    let mut framebuffer = 0;
    let mut texture_color_buffer = 0;
    unsafe {
        gl::GenFramebuffers(1, &mut framebuffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);

        // create a color attachment texture
        gl::GenTextures(1, &mut texture_color_buffer);

        // "Bind" the newly created texture : all future texture functions will modify this texture
        gl::BindTexture(gl::TEXTURE_2D, texture_color_buffer);
        // Give an empty image to OpenGL ( the last "0" )
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            texture_dims.0,
            texture_dims.1,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            std::ptr::null(),
        );
        // Poor filtering. Needed !
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        // Set "renderedTexture" as our colour attachement #0
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            texture_color_buffer,
            0,
        );
        // create a renderbuffer object for depth and stencil attachment (we won't be sampling these)
        let mut rbo = 0;
        gl::GenRenderbuffers(1, &mut rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
        gl::RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH24_STENCIL8,
            texture_dims.0,
            texture_dims.1,
        ); // use a single renderbuffer object for both a depth AND stencil buffer.
        gl::FramebufferRenderbuffer(
            gl::FRAMEBUFFER,
            gl::DEPTH_STENCIL_ATTACHMENT,
            gl::RENDERBUFFER,
            rbo,
        ); // now actually attach it
           // now that we actually created the framebuffer and added all attachments we want to check if it is actually complete now
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    TestMask {
        ndc_quad_vao: VAO::new(&ndc_quad_mesh, &shader.id),
        shader: shader,
        _ndc_quad_mesh: ndc_quad_mesh,
        framebuffer,
        texture_buffer: texture_color_buffer,
        texture_dims,
    }
}

/*
fn renderpass() {
    // MOUSE MASK pass
    let test = ecs.get_resource::<TestMask>().unwrap();
    let mouse = ecs.get_resource::<CursorRaycast>().unwrap();

    println!("mouse: {}", mouse.0);

    gl::Viewport(0, 0, test.texture_dims.0, test.texture_dims.1);

    gl::BindFramebuffer(gl::FRAMEBUFFER, test.framebuffer);

    // TODO: dont clear this! and enable MAX blending, this way you can accummulate this pass (and only render it if the mouse is pressed)
    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

    test.shader.gl_use_program();
    gl::BindVertexArray(test.ndc_quad_vao.id());

    test.shader
        .set_gl_uniform("Mouse_Position", GlUniform::Vec3(mouse.0.to_array()));

    gl::DrawElements(
        gl::TRIANGLES,
        test.ndc_quad_vao.indices_count as i32,
        gl::UNSIGNED_INT,
        ptr::null(),
    );

    let texture_buffer = test.texture_buffer;
}
*/
