use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};
use windows as ws;

pub fn init_logs(log_level: LevelFilter) {
    let _ = unsafe { ws::Win32::System::Console::AllocConsole() };
    let config = ConfigBuilder::new().set_time_format_rfc2822().build();

    let _ = TermLogger::init(log_level, config, TerminalMode::Mixed, ColorChoice::Auto);
}
