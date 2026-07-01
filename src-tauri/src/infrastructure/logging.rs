pub fn init() {
    let _ = tracing_subscriber::fmt()
        .with_target(false)
        .without_time()
        .try_init();
}
