use glutin::event_loop::EventLoop;
use glutin::window::{Window, WindowBuilder};
use glutin::{
    dpi::{PhysicalSize, Size},
    ContextBuilder,
};
use glutin::{ContextWrapper, GlProfile, PossiblyCurrent};

pub fn setup_glutin_and_opengl(
    window_size: (u32, u32),
) -> (ContextWrapper<PossiblyCurrent, Window>, EventLoop<()>) {
    let el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("Country Slice")
        .with_inner_size(Size::Physical(PhysicalSize::new(
            window_size.0,
            window_size.1,
        )));

    // https://docs.rs/glutin/0.7.4/glutin/struct.WindowBuilder.html
    // look into with_depth_buffer() -> see if its solves some of the z-fighting in case there is not enough precision

    let windowed_context = ContextBuilder::new()
        .with_gl_profile(GlProfile::Core)
        .with_vsync(true)
        .with_multisampling(4)
        .build_windowed(wb, &el)
        .unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    println!(
        "Pixel format of the window's GL context: {:?}",
        windowed_context.get_pixel_format()
    );

    // gl: load all OpenGL function pointers
    // ---------------------------------------
    gl::load_with(|ptr| windowed_context.context().get_proc_address(ptr) as *const _);

    // There is a maximum number of vertex attributes we're allowed to declare limited by the hardware.
    // OpenGL guarantees there are always at least 16 4-component vertex attributes available
    unsafe {
        let mut max_vertex_atribbs = 0;
        gl::GetIntegerv(gl::MAX_VERTEX_ATTRIBS, &mut max_vertex_atribbs);
        //println!(
        //    "Maximum nr of vertex attributes supported: {}",
        //    max_vertex_atribbs
        //);
    }

    // Setup OpenGL flags
    unsafe {
        // uncomment this call to draw in wireframe polygons.
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        //gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

        gl::Enable(gl::DEPTH_TEST);
        // enabled outputting linear color in shaders
        gl::Enable(gl::FRAMEBUFFER_SRGB);

        // enable transparency
        gl::Enable(gl::BLEND);

        gl::BlendFuncSeparate(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA, gl::ONE, gl::ONE);
        gl::BlendEquationSeparate(gl::FUNC_ADD, gl::FUNC_ADD);
    }

    (windowed_context, el)
}
