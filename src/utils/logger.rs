pub fn init_logger() {
    use chrono::Local;
    use fern::Dispatch;
    use log::LevelFilter;

    // fn get_executable_name() -> String {
    //     use std::env;
    //     env::current_exe()
    //         .ok()
    //         .and_then(|path| path.file_stem().map(|os_str| os_str.to_string_lossy().into_owned()))
    //         .unwrap_or_else(|| "unknown".to_string())
    // }

    // let executable_name = get_executable_name();
    // let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();

    // Compose filename with timestamp
    // let log_path = format!("{}.log", executable_name);

    // let log_file = std::fs::OpenOptions::new()
    //     .create(true)
    //     .write(true)
    //     .append(true)
    //     .open(&log_path)
    //     .expect("Failed to open log file");

    // let file_dispatch = Dispatch::new().level(LevelFilter::Trace).chain(log_file);

    let console_dispatch = Dispatch::new().level(LevelFilter::Debug).chain(std::io::stderr());

    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        // .chain(file_dispatch)
        .chain(console_dispatch)
        .apply()
        .expect("Failed to initialize logger");

    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("Panic occurred: {}", panic_info);
    }));
}
