use log::debug;
use std::cell;
use windows::{
    self as ws,
    Win32::System::{
        IO::OVERLAPPED,
        RemoteDesktop::{CHANNEL_FLAG_FIRST, CHANNEL_FLAG_LAST},
    },
};

pub fn write_dvc(
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
        } else {
            return Err(err);
        }
    }

    let data_str = String::from_utf8_lossy(data);

    debug!("written: {real_written} bytes");
    debug!("sent: {data_str} ({data:?})");
    Ok(())
}

pub fn read_dvc(
    filehandle: ws::Win32::Foundation::HANDLE,
    data: &mut [u8],
    ref_overlapped: &cell::RefCell<OVERLAPPED>,
) -> Result<std::ops::Range<usize>, ws::core::Error> {
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
            debug!("read GetOverlappedResult");
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
                Err(err) => return Err(err),
            };
        } else {
            return Err(err);
        }
    }

    if real_read < 8 {
        return Err(ws::core::Error::new(
            ws::Win32::Foundation::E_FAIL,
            format!("not a PDU header (length = {real_read}): {data:?}"),
        ));
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
        return Err(ws::core::Error::new(
            ws::Win32::Foundation::E_FAIL,
            format!("unsupported PDU flags: 0x{pdu_flags:x}"),
        ));
    }

    if pdu_length != real_read - 8 {
        return Err(ws::core::Error::new(
            ws::Win32::Foundation::E_FAIL,
            format!("inconsistent length: pdu_length = {pdu_length} - read = {real_read}"),
        ));
    }

    Ok(8..real_read as usize)
}
