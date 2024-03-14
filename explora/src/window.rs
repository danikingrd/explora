use std::{sync::Arc, time::Instant};

use crate::{key_state::KeyState, render::Renderer, scene::Scene};
use common::math::Vec2;
use winit::{
    event::{DeviceEvent, Event, KeyEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window as Platform, WindowBuilder},
};

pub struct Window {
    platform: Arc<Platform>,
    event_loop: Option<EventLoop<()>>,
    renderer: Renderer,
    scene: Scene,
    cursor_grabbed: bool,
}

impl Window {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        event_loop.set_control_flow(ControlFlow::Poll);

        let window = WindowBuilder::new()
            .with_title("explora")
            .build(&event_loop)
            .unwrap();

        let window = Arc::new(window);

        let renderer = Renderer::new(&window);

        let size = window.inner_size();
        let scene = Scene::new(size.width as f32 / size.height as f32);

        Self {
            platform: window,
            event_loop: Some(event_loop),
            renderer,
            scene,
            cursor_grabbed: false,
        }
    }

    pub fn run(&mut self) {
        tracing::info!("Running explora...");
        let mut key_state = KeyState::default();
        let mut last_frame = Instant::now();
        const SENSITIVITY: f32 = 100.0;
        self.event_loop
            .take()
            .unwrap()
            .run(move |event, elwt| match event {
                Event::WindowEvent { window_id, event } if window_id == self.platform.id() => {
                    match event {
                        winit::event::WindowEvent::Resized(size) => {
                            self.renderer.resize(size.width, size.height);
                            self.scene.resize(size.width as f32, size.height as f32);
                        }
                        winit::event::WindowEvent::CloseRequested => {
                            tracing::info!("Application quit requested.");
                            elwt.exit();
                        }
                        winit::event::WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state,
                                    physical_key: PhysicalKey::Code(code),
                                    ..
                                },
                            ..
                        } => {
                            key_state.update(code, state.is_pressed());
                            if matches!(code, winit::keyboard::KeyCode::Escape)
                                && state.is_pressed()
                            {
                                self.grab_cursor(!self.cursor_grabbed);
                            }
                        }
                        _ => (),
                    }
                }
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta: (dx, dy) },
                    ..
                } => {
                    // map sensitivity to a range of 1 - 200. 100 being default.
                    let delta = Vec2::new(
                        dx as f32 * (SENSITIVITY / 100.0),
                        dy as f32 * (SENSITIVITY / 100.0),
                    );
                    self.scene.look(delta.x, delta.y);
                }
                Event::AboutToWait => {
                    let dt = last_frame.elapsed();
                    self.scene.set_movement_dir(key_state.dir());
                    self.scene.tick(dt.as_secs_f32());
                    last_frame = Instant::now();
                    self.renderer.render(&mut self.scene);
                }
                _ => (),
            })
            .unwrap();
    }

    pub fn grab_cursor(&mut self, value: bool) {
        self.platform.set_cursor_visible(!value);
        let mode = if value {
            winit::window::CursorGrabMode::Locked
        } else {
            winit::window::CursorGrabMode::None
        };
        match self.platform.set_cursor_grab(mode) {
            Ok(_) => self.cursor_grabbed = value,
            Err(e) => tracing::warn!("Could not grab cursor in {:?} mode ({})", mode, e),
        }
    }
}
