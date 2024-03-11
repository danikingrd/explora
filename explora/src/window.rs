use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct Window {
    platform: winit::window::Window,
    event_loop: Option<EventLoop<()>>,
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
        Self {
            platform: window,
            event_loop: Some(event_loop),
        }
    }

    pub fn run(&mut self) {
        tracing::info!("Running explora...");
        self.event_loop
            .take()
            .unwrap()
            .run(move |event, elwt| {
                match event {
                    Event::WindowEvent { window_id, event } if window_id == self.platform.id() => {
                        match event {
                            winit::event::WindowEvent::Resized(size) => {}
                            winit::event::WindowEvent::CloseRequested => {
                                tracing::info!("Application quit requested.");
                                elwt.exit();
                            }
                            _ => (),
                        }
                    }
                    Event::AboutToWait => {
                        // Application update code.

                        // Queue a RedrawRequested event.
                        //
                        // You only need to call this if you've determined that you need to redraw in
                        // applications which do not always need to. Applications that redraw continuously
                        // can render here instead.
                        self.platform.request_redraw();
                    }
                    Event::WindowEvent {
                        event: WindowEvent::RedrawRequested,
                        ..
                    } => {
                        // Redraw the application.
                        //
                        // It's preferable for applications that do not render continuously to render in
                        // this event rather than in AboutToWait, since rendering in here allows
                        // the program to gracefully handle redraws requested by the OS.
                    }
                    _ => (),
                }
            })
            .unwrap();
    }
}
