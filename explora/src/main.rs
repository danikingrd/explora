use explora::window::Window;

fn main() {
    common_log::init();
    let mut window = Window::new();
    window.grab_cursor(true);
    window.run();
}
