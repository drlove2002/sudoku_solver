use fern::Dispatch;
use log::LevelFilter;

pub fn init_logger() {
    Dispatch::new()
        // Terminal output with simpler format
        .chain(
            Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!("[{}] {}", record.level(), message))
                })
                .level(LevelFilter::Info)
                .chain(std::io::stdout()),
        )
        // File output with detailed format
        .chain(
            Dispatch::new()
                .format(|out, message, record| {
                    let file = record.file().unwrap_or("unknown");
                    let filename = std::path::Path::new(file)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(file);

                    out.finish(format_args!(
                        "{} [{}] ({}:{}) {}",
                        chrono::Local::now().format("%H:%M:%S"),
                        record.level(),
                        filename,
                        record.line().unwrap_or(0),
                        message
                    ))
                })
                .level(LevelFilter::Trace)
                .filter(|metadata| {
                    metadata.level() == log::Level::Debug || metadata.level() == log::Level::Trace
                })
                .chain(fern::log_file("trace.log").expect("Failed to create log file")),
        )
        .apply()
        .expect("Failed to initialize logger");
}
