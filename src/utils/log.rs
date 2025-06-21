#[ctor::ctor]
fn init_logging() {
    use chrono::Local;
    use fern::Dispatch;

    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();

    // Compose filename with timestamp
    let log_path = format!("app_{}.log", timestamp);

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
        .expect("Failed to open log file");

    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stderr())
        .chain(log_file)
        .apply()
        .expect("Failed to initialize logger");

    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("Panic occurred: {}", panic_info);
    }));

    log::info!("Logging initialized. Log file: {}", log_path);
}
