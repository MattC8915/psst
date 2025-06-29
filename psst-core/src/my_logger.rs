use chrono;
use env_logger::{Builder, Env};
use log;
use std::io::Write;

/// Initialize the centralized logging configuration used by both psst-cli and psst-gui.
/// 
/// This function sets up a custom logger with:
/// - Colored log levels (ERROR, WARN, INFO, DEBUG, TRACE)
/// - Timestamp format: YYYY-MM-DD HH:MM:SS.mmm
/// - Default log level: "info"
/// - Always enabled colors
pub fn init_logging() {
    Builder::from_env(Env::default().default_filter_or("info"))
        .write_style(env_logger::WriteStyle::Always)
        .format(|buf, record| {
            let level = record.level();
            let target = record.target();
            let args = record.args();
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");

            // Manually color the level text
            let level_str = match level {
                log::Level::Error => "\x1b[1;31mERROR\x1b[0m", // bold red
                log::Level::Warn  => "\x1b[1;33mWARN\x1b[0m",  // bold yellow
                log::Level::Info  => "\x1b[1;32mINFO\x1b[0m",  // bold green
                log::Level::Debug => "\x1b[1;36mDEBUG\x1b[0m", // bold cyan
                log::Level::Trace => "\x1b[1;37mTRACE\x1b[0m", // bold white
            };

            writeln!(
                buf,
                "{} {} [{}] {}",
                timestamp,
                level_str,
                target,
                args
            )
        })
        .init();
}

/// Initialize logging with custom environment variable names.
/// 
/// This function allows specifying custom environment variable names for log level
/// and log style configuration.
pub fn init_logging_with_env(log_env: &str, style_env: &str) {
    Builder::from_env(
        Env::new()
            .filter_or(log_env, "info")
            .write_style(style_env),
    )
    .write_style(env_logger::WriteStyle::Always)
    .format(|buf, record| {
        let level = record.level();
        let target = record.target();
        let args = record.args();
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");

        // Manually color the level text
        let level_str = match level {
            log::Level::Error => "\x1b[1;31mERROR\x1b[0m", // bold red
            log::Level::Warn  => "\x1b[1;33mWARN\x1b[0m",  // bold yellow
            log::Level::Info  => "\x1b[1;32mINFO\x1b[0m",  // bold green
            log::Level::Debug => "\x1b[1;36mDEBUG\x1b[0m", // bold cyan
            log::Level::Trace => "\x1b[1;37mTRACE\x1b[0m", // bold white
        };

        writeln!(
            buf,
            "{} {} [{}] {}",
            timestamp,
            level_str,
            target,
            args
        )
    })
    .init();
} 