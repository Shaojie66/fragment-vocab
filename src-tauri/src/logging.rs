use log::LevelFilter;
use simplelog::{
    CombinedLogger, ConfigBuilder, SharedLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::fs;
use std::path::PathBuf;

const MAX_LOG_FILES: usize = 7;

fn log_dir(app_data_dir: &PathBuf) -> PathBuf {
    app_data_dir.join("logs")
}

fn today_log_path(app_data_dir: &PathBuf) -> PathBuf {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    log_dir(app_data_dir).join(format!("fragment-vocab-{}.log", date))
}

/// Delete log files older than MAX_LOG_FILES days
fn cleanup_old_logs(app_data_dir: &PathBuf) {
    let dir = log_dir(app_data_dir);
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };

    let mut log_files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().map_or(false, |ext| ext == "log")
                && p.file_name().map_or(false, |n| {
                    n.to_string_lossy().starts_with("fragment-vocab-")
                })
        })
        .collect();

    if log_files.len() <= MAX_LOG_FILES {
        return;
    }

    log_files.sort();
    let to_remove = log_files.len() - MAX_LOG_FILES;
    for path in log_files.into_iter().take(to_remove) {
        let _ = fs::remove_file(path);
    }
}

pub fn init(app_data_dir: &PathBuf) {
    let dir = log_dir(app_data_dir);
    let _ = fs::create_dir_all(&dir);

    cleanup_old_logs(app_data_dir);

    let config = ConfigBuilder::new().set_time_format_rfc3339().build();

    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![TermLogger::new(
        LevelFilter::Info,
        config.clone(),
        TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )];

    if let Ok(file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(today_log_path(app_data_dir))
    {
        loggers.push(WriteLogger::new(LevelFilter::Debug, config, file));
    }

    let _ = CombinedLogger::init(loggers);
}
