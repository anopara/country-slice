use std::convert::TryInto;

use bevy_app::Events;
use bevy_input::mouse::MouseButtonInput;
use glam::{Mat4, Vec2};
use glutin::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use glutin::event_loop::ControlFlow;
use glutin::window::Window;

use glutin::{ContextWrapper, PossiblyCurrent};

use crate::render::camera::MainCamera;

use crate::render_loop::render;

// Bevy Events

pub struct CursorMoved {
    pub pos: Vec2,
}

pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl WindowSize {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            width: w,
            height: h,
        }
    }

    pub fn set(&mut self, w: u32, h: u32) {
        self.width = w;
        self.height = h;
    }

    pub fn try_into_i32(&self) -> (i32, i32) {
        (
            self.width.try_into().unwrap(),
            self.height.try_into().unwrap(),
        )
    }
}

// ------

pub fn process_window_events(
    event: Event<()>,
    windowed_context: &mut ContextWrapper<PossiblyCurrent, Window>,
    control_flow: &mut ControlFlow,
    app: &mut bevy_app::AppBuilder,
) {
    let ecs = app.world_mut();

    match event {
        Event::LoopDestroyed => return,
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::Resized(physical_size) => {
                windowed_context.resize(physical_size);

                // TODO: move camera update to a system?
                let mut main_camera = ecs.get_resource_mut::<MainCamera>().unwrap();
                main_camera.camera.perspective_projection = Mat4::perspective_rh_gl(
                    (45.0_f32).to_radians(),
                    physical_size.width as f32 / physical_size.height as f32,
                    0.1,
                    100.0,
                );
                unsafe {
                    gl::Viewport(
                        0,
                        0,
                        physical_size.width as i32,
                        physical_size.height as i32,
                    )
                }
                let mut window_size = ecs.get_resource_mut::<WindowSize>().unwrap();
                window_size.set(physical_size.width, physical_size.height);
            }
            // If window was closed, or we pressed "Escape", close the app
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(virtual_code),
                        state,
                        scancode,
                        ..
                    },
                ..
            } => {
                if VirtualKeyCode::Escape == virtual_code {
                    *control_flow = ControlFlow::Exit;
                }

                // TODO: do this with marcos?
                let temp = match virtual_code {
                    VirtualKeyCode::Left => Some(bevy_input::keyboard::KeyCode::Left),
                    VirtualKeyCode::Right => Some(bevy_input::keyboard::KeyCode::Right),
                    VirtualKeyCode::Up => Some(bevy_input::keyboard::KeyCode::Up),
                    VirtualKeyCode::Down => Some(bevy_input::keyboard::KeyCode::Down),
                    VirtualKeyCode::Space => Some(bevy_input::keyboard::KeyCode::Space),
                    VirtualKeyCode::Q => Some(bevy_input::keyboard::KeyCode::Q),
                    VirtualKeyCode::E => Some(bevy_input::keyboard::KeyCode::E),
                    VirtualKeyCode::Escape => Some(bevy_input::keyboard::KeyCode::Escape),
                    VirtualKeyCode::Back => Some(bevy_input::keyboard::KeyCode::Back),
                    VirtualKeyCode::Key1 => Some(bevy_input::keyboard::KeyCode::Key1),
                    VirtualKeyCode::Key2 => Some(bevy_input::keyboard::KeyCode::Key2),
                    _ => None,
                };

                let event = bevy_input::keyboard::KeyboardInput {
                    key_code: temp,
                    state: match state {
                        ElementState::Pressed => bevy_input::ElementState::Pressed,
                        ElementState::Released => bevy_input::ElementState::Released,
                    },
                    scan_code: scancode,
                };
                let mut keyboard_events = ecs
                    .get_resource_mut::<Events<bevy_input::keyboard::KeyboardInput>>()
                    .unwrap();
                keyboard_events.send(event);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let mut cursor_events = ecs.get_resource_mut::<Events<CursorMoved>>().unwrap();
                cursor_events.send(CursorMoved {
                    pos: Vec2::new(position.x as f32, position.y as f32),
                });
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let mut mouse_events = ecs.get_resource_mut::<Events<MouseButtonInput>>().unwrap();

                let button = match button {
                    glutin::event::MouseButton::Left => bevy_input::mouse::MouseButton::Left,
                    glutin::event::MouseButton::Right => bevy_input::mouse::MouseButton::Right,
                    glutin::event::MouseButton::Middle => bevy_input::mouse::MouseButton::Middle,
                    glutin::event::MouseButton::Other(val) => {
                        bevy_input::mouse::MouseButton::Other(val)
                    }
                };

                mouse_events.send(MouseButtonInput {
                    button,
                    state: match state {
                        ElementState::Pressed => bevy_input::ElementState::Pressed,
                        ElementState::Released => bevy_input::ElementState::Released,
                    },
                });
            }
            _ => (),
        },
        Event::RedrawRequested(_) => render(ecs, windowed_context),
        Event::MainEventsCleared => {
            puffin::profile_scope!("main_loop");
            puffin::GlobalProfiler::lock().new_frame();

            // Application update code.
            app.app.update();
            windowed_context.window().request_redraw();
        }
        _ => (),
    }
}
