use std::{
    cell,
    fmt::Display,
    io::{self, Write},
    process::exit,
    ptr,
};

use clap::Parser;

use log::{debug, error};
use simplelog::Config;
use windows::{
    self as ws,
    Win32::System::{
        IO::OVERLAPPED,
        RemoteDesktop::{
            CHANNEL_CHUNK_LENGTH, CHANNEL_FLAG_FIRST, CHANNEL_FLAG_LAST,
            WTS_CHANNEL_OPTION_DYNAMIC, WTS_CURRENT_SESSION, WTSVirtualChannelOpenEx,
            WTSVirtualChannelQuery,
        },
    },
    core::PCSTR,
};

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

fn main() {
    let opts = Cli::parse();

    let level = if opts.verbose {
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

    let channel_name = opts.name;

    println!("opening channel: {}", channel_name);

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

    println!("{HELP_MSG}");
    let mut input = String::new();
    loop {
        print!("{PROMPT}");
        io::stdout().flush().unwrap();

        let _ = io::stdin().read_line(&mut input).unwrap();
        let line = input.trim();

        let (command, arg) = line
            .split_once(' ')
            .map(|(command, arg)| (command, arg))
            .unwrap_or((line, ""));

        let command = command.to_uppercase();

        debug!("command: {command}");
        debug!("arg: {arg}");

        match command.as_str() {
            "" => (),
            "QUIT" | "EXIT" => break,
            "WRITE" | "PUT" => {
                let _ =
                    write_dvc(filehandle, arg.as_bytes(), &write_overlapped).inspect_err(|err| {
                        error!("error writting to channel: {err}");
                        exit(1)
                    });
                let mut rbuf: [u8; CHANNEL_CHUNK_LENGTH as usize] =
                    [0; CHANNEL_CHUNK_LENGTH as usize];
                let data_range = match read_dvc(filehandle, &mut rbuf, &read_overlapped) {
                    Ok(r) => r,
                    Err(err) => {
                        error!("error reading from channel: {err}");
                        exit(1)
                    }
                };

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
}

fn write_dvc(
    filehandle: ws::Win32::Foundation::HANDLE,
    data: &[u8],
    ref_overlapped: &cell::RefCell<OVERLAPPED>,
) -> ws::core::Result<()> {
    let mut written = 0;

    let mut overlapped = ref_overlapped.borrow_mut();

    debug!("WriteFile");
    let ret = unsafe {
        ws::Win32::Storage::FileSystem::WriteFile(
            filehandle,
            Some(data),
            Some(&raw mut written),
            Some(&raw mut *overlapped),
        )
    };

    let mut real_written = written;
    if let Err(err) = ret {
        if err.code() == ws::Win32::Foundation::ERROR_IO_PENDING.to_hresult() {
            let mut written = 0;
            let ret = unsafe {
                ws::Win32::System::IO::GetOverlappedResult(
                    filehandle,
                    &raw const *overlapped,
                    &raw mut written,
                    true,
                )
            };

            real_written = match ret {
                Ok(_) => written,
                Err(err) => return Err(err),
            };
        }
    }

    let data_str = String::from_utf8_lossy(data);

    debug!("written: {real_written} bytes");
    debug!("sent: {data_str} ({data:?})");
    Ok(())
}

struct ReadError(String);

impl From<ws::core::Error> for ReadError {
    fn from(value: ws::core::Error) -> Self {
        Self(format!("{value}"))
    }
}

impl Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn read_dvc(
    filehandle: ws::Win32::Foundation::HANDLE,
    data: &mut [u8],
    ref_overlapped: &cell::RefCell<OVERLAPPED>,
) -> Result<std::ops::Range<usize>, ReadError> {
    let mut read = 0;
    let mut overlapped = ref_overlapped.borrow_mut();

    debug!("ReadFile");
    let ret = unsafe {
        ws::Win32::Storage::FileSystem::ReadFile(
            filehandle,
            Some(data),
            Some(&raw mut read),
            Some(&raw mut *overlapped),
        )
    };

    let mut real_read = read;
    if let Err(err) = ret {
        if err.code() == ws::Win32::Foundation::ERROR_IO_PENDING.to_hresult() {
            let mut read = 0;
            let ret = unsafe {
                ws::Win32::System::IO::GetOverlappedResult(
                    filehandle,
                    &raw const *overlapped,
                    &raw mut read,
                    true,
                )
            };

            real_read = match ret {
                Ok(_) => read,
                Err(err) => return Err(ReadError::from(err)),
            };
        }
    }

    if real_read < 8 {
        return Err(ReadError(format!(
            "not a PDU header (length = {real_read}): {data:?}"
        )));
    }

    let mut pdu_length = [0u8; 4];
    pdu_length.copy_from_slice(&data[..4]);
    let pdu_length = u32::from_le_bytes(pdu_length);

    let mut pdu_flags = [0u8; 4];
    pdu_flags.copy_from_slice(&data[4..8]);
    let pdu_flags = u32::from_le_bytes(pdu_flags);

    if pdu_flags & (CHANNEL_FLAG_FIRST | CHANNEL_FLAG_LAST)
        != CHANNEL_FLAG_FIRST | CHANNEL_FLAG_LAST
    {
        return Err(ReadError(format!("unsupported PDU flags: 0x{pdu_flags:x}")));
    }

    if pdu_length != real_read - 8 {
        return Err(ReadError(format!(
            "inconsistent length: pdu_length = {pdu_length} - read = {real_read}"
        )));
    }

    Ok(8..real_read as usize)
}
