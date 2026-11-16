fn main() {
    tracing_subscriber::fmt()
        .without_time()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let buffer =
        std::fs::read("submodules/pspautotests/demos/HelloJpcsp.pbp").expect("Failed to read file");

    let mut psp = core::Psp::new(&buffer);
    psp.run();
}
