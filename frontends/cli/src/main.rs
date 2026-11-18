mod app;

fn main() {
    tracing_subscriber::fmt()
        .without_time()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let path = std::env::args().nth(1).expect("No path provided");
    let buffer = std::fs::read(&path).expect("Failed to read file");

    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop
        .run_app(&mut app::Emulator::new(&buffer))
        .expect("Event loop failed");
}
