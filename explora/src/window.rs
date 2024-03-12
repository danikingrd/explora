use std::sync::Arc;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window as Platform, WindowBuilder},
};

use crate::renderer::Renderer;

pub struct Window {
    platform: Arc<Platform>,
    event_loop: Option<EventLoop<()>>,
    renderer: Renderer,
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

        Self {
            platform: window,
            event_loop: Some(event_loop),
            renderer,
        }
    }

    pub fn run(&mut self) {
        tracing::info!("Running explora...");
        self.event_loop
            .take()
            .unwrap()
            .run(move |event, elwt| match event {
                Event::WindowEvent { window_id, event } if window_id == self.platform.id() => {
                    match event {
                        winit::event::WindowEvent::Resized(size) => {
                            self.renderer.resize(size.width, size.height);
                        }
                        winit::event::WindowEvent::CloseRequested => {
                            tracing::info!("Application quit requested.");
                            elwt.exit();
                        }
                        _ => (),
                    }
                }
                Event::AboutToWait => {
                    self.renderer.render();
                }
                _ => (),
            })
            .unwrap();
    }
}
