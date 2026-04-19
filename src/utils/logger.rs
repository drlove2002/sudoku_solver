use fern::Dispatch;
use log::LevelFilter;

pub fn init_logger() {
    let mut dispatch = Dispatch::new().chain(
        Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!("[{}] {}", record.level(), message))
            })
            .level(LevelFilter::Info)
            .chain(std::io::stdout()),
    );

    #[cfg(debug_assertions)]
    {
        dispatch = dispatch.chain(
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
                .chain(
                    std::fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open("trace.log")
                        .expect("Failed to create log file"),
                ),
        );
    }

    dispatch.apply().expect("Failed to initialize logger");
}
