use std::path::Path;
use std::{fs::File, path::PathBuf};

use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, SharedLogger, TermLogger,
    TerminalMode, WriteLogger,
};
use windows as ws;

pub fn init_logs<P>(log_level: LevelFilter, filepath: P)
where
    P: AsRef<Path>,
{
    let _ = unsafe { ws::Win32::System::Console::AllocConsole() };
    let config = ConfigBuilder::new().set_time_format_rfc2822().build();

    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![TermLogger::new(
        log_level,
        config.clone(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )];

    let mut path = PathBuf::from("C:\\Users\\citrix\\Desktop");
    path.push(filepath);

    if let Ok(file) = File::options()
        .create(true)
        .append(false)
        .truncate(true)
        .write(true)
        .open(path)
    {
        loggers.push(WriteLogger::new(log_level, config, file));
    }

    let _ = CombinedLogger::init(loggers);
}
