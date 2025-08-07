mod io_dvc;

use std::{
    cell,
    io::{self, Write},
    process::exit,
    ptr,
};

use clap::Parser;
use io_dvc::{read_dvc, write_dvc};

use log::{debug, error};
use simplelog::Config;
use windows::{
    self as ws,
    Win32::System::{
        IO::OVERLAPPED,
        RemoteDesktop::{
            CHANNEL_CHUNK_LENGTH, WTS_CHANNEL_OPTION_DYNAMIC, WTS_CURRENT_SESSION,
            WTSVirtualChannelOpenEx, WTSVirtualChannelQuery,
        },
    },
    core::PCSTR,
};

const PDU_HEADER_LENGTH: usize = 0x8;
const PACKET_MAX_LENGTH: usize = CHANNEL_CHUNK_LENGTH as usize + PDU_HEADER_LENGTH;
const DVC_NAME_DEFAULT: &str = "ECHOCHN";

const HELP_MSG: &str = r#"
Usage:
- "write XXXX" or "put XXXX" to write to the DVC
- "quit" or "exit" to leave this interface
"#;
const PROMPT: &str = "echo_dvc> ";

#[derive(Parser)]
#[command(name = "echo_dvc_server")]
struct Cli {
    #[arg(short, long, help = "enable debug logs")]
    verbose: bool,
    #[arg(default_value = DVC_NAME_DEFAULT, help = "DVC name to open")]
    name: String,
}

fn init_logs(verbose: bool) {
    let level = if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Error
    };

    let _ = simplelog::TermLogger::init(
        level,
        Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    );
}

fn main() {
    let opts = Cli::parse();
    init_logs(opts.verbose);

    let channel_name = opts.name;

    println!("opening channel: {channel_name}");

    let ch_handle = match unsafe {
        WTSVirtualChannelOpenEx(
            WTS_CURRENT_SESSION,
            PCSTR(format!("{channel_name}\0").as_ptr()),
            WTS_CHANNEL_OPTION_DYNAMIC,
        )
    } {
        Ok(handle) => handle,
        Err(err) => {
            error!("failed to open DVC {channel_name}: {err}");
            error!("Are you sure the plugin is correctly loaded ?");
            exit(1);
        }
    };

    if ch_handle.0.is_null() {
        let err = io::Error::last_os_error();
        error!("error channel handle is null: {err}");
        exit(1);
    }

    debug!("channel handle ok: {ch_handle:?}");

    let mut filehandleptr: *mut ws::Win32::Foundation::HANDLE = ptr::null_mut();
    let filehandleptrptr: *mut *mut ws::Win32::Foundation::HANDLE = &raw mut filehandleptr;
    let mut len = 0;

    debug!("WTSVirtualChannelQuery");
    let ret = unsafe {
        WTSVirtualChannelQuery(
            ch_handle,
            ws::Win32::System::RemoteDesktop::WTSVirtualFileHandle,
            filehandleptrptr.cast(),
            &raw mut len,
        )
    };

    match ret {
        Ok(_) => {}
        Err(err) => {
            error!("WTSVirtualChannelQuery failed: {err}");
            exit(1);
        }
    }

    if filehandleptr.is_null() {
        let err = io::Error::last_os_error();
        error!("error file handle is null: {err}");
        exit(1);
    }

    let filehandle = unsafe { *filehandleptr };
    debug!("filehandle: {filehandle:?}");

    let h_event = unsafe {
        ws::Win32::System::Threading::CreateEventA(Some(ptr::null()), false, false, PCSTR::null())
    }
    .unwrap();

    if h_event.0.is_null() {
        let err = io::Error::last_os_error();
        error!("error handle event is null: {err}");
        exit(1);
    }

    let anonymous = ws::Win32::System::IO::OVERLAPPED_0 {
        Pointer: ptr::null_mut(),
    };

    let read_overlapped = ws::Win32::System::IO::OVERLAPPED {
        Internal: 0,
        InternalHigh: 0,
        Anonymous: anonymous,
        hEvent: h_event,
    };

    let write_overlapped = ws::Win32::System::IO::OVERLAPPED {
        Internal: 0,
        InternalHigh: 0,
        Anonymous: anonymous,
        hEvent: ws::Win32::Foundation::HANDLE(ptr::null_mut()),
    };

    let read_overlapped = cell::RefCell::new(read_overlapped);
    let write_overlapped = cell::RefCell::new(write_overlapped);

    match run(filehandle, read_overlapped, write_overlapped) {
        Ok(_) => {}
        Err(e) => {
            error!("error: {e}");
            exit(1);
        }
    }
}

fn run(
    filehandle: ws::Win32::Foundation::HANDLE,
    read_overlapped: cell::RefCell<OVERLAPPED>,
    write_overlapped: cell::RefCell<OVERLAPPED>,
) -> ws::core::Result<()> {
    println!("{HELP_MSG}");
    let mut input = String::new();
    loop {
        print!("{PROMPT}");
        io::stdout().flush().unwrap();

        let _ = io::stdin().read_line(&mut input).unwrap();
        let line = input.trim();

        let (command, arg) = line.split_once(' ').unwrap_or((line, ""));

        let command = command.to_uppercase();

        debug!("command: {command}");
        debug!("arg: {arg}");

        match command.as_str() {
            "" => (),
            "QUIT" | "EXIT" => break,
            "WRITE" | "PUT" => {
                // Send
                write_dvc(filehandle, arg.as_bytes(), &write_overlapped).map_err(|err| {
                    ws::core::Error::new(
                        err.code(),
                        format!("error writting to channel: {}", err.message()),
                    )
                })?;

                // Receive
                let mut rbuf: [u8; PACKET_MAX_LENGTH] = [0; PACKET_MAX_LENGTH];
                let data_range =
                    read_dvc(filehandle, &mut rbuf, &read_overlapped).map_err(|err| {
                        ws::core::Error::new(
                            err.code(),
                            format!("error reading from channel: {}", err.message()),
                        )
                    })?;

                println!(
                    "received: {} ({:?})",
                    String::from_utf8_lossy(&rbuf[data_range.clone()]),
                    &rbuf[data_range]
                );
            }
            _ => println!("invalid command"),
        }

        input.clear();
    }

    Ok(())
}
