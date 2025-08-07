use log::debug;
use std::cell;
use windows::{
    self as ws,
    Win32::System::{
        IO::OVERLAPPED,
        RemoteDesktop::{CHANNEL_FLAG_FIRST, CHANNEL_FLAG_LAST, CHANNEL_FLAG_MIDDLE},
    },
};

use crate::{PACKET_MAX_LENGTH, PDU_HEADER_LENGTH};

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
    ref_overlapped: &cell::RefCell<OVERLAPPED>,
) -> Result<String, ws::core::Error> {
    let mut tot_read = 0;
    let mut specified_pdu_length = 0;

    let mut read_string = String::new();
    let mut overlapped = ref_overlapped.borrow_mut();
    loop {
        debug!("ReadFile");
        let mut rbuf = [0u8; PACKET_MAX_LENGTH];
        let mut read = 0;
        let ret = unsafe {
            ws::Win32::Storage::FileSystem::ReadFile(
                filehandle,
                Some(&mut rbuf),
                Some(&raw mut read),
                Some(&raw mut *overlapped),
            )
        };

        let mut real_read = read;
        if let Err(err) = ret {
            if err.code() == ws::Win32::Foundation::ERROR_IO_PENDING.to_hresult() {
                debug!("read GetOverlappedResult");
                let mut overlapped_read = 0;
                let ret = unsafe {
                    ws::Win32::System::IO::GetOverlappedResult(
                        filehandle,
                        &raw const *overlapped,
                        &raw mut overlapped_read,
                        true,
                    )
                };

                real_read = match ret {
                    Ok(_) => overlapped_read,
                    Err(err) => return Err(err),
                };
            } else {
                return Err(err);
            }
        }

        if real_read < 8 {
            return Err(ws::core::Error::new(
                ws::Win32::Foundation::E_FAIL,
                format!("not a PDU header (length = {real_read}): {rbuf:?}"),
            ));
        }

        // Parse CHANNEL_PDU_HEADER
        let mut pdu_length = [0u8; 4];
        pdu_length.copy_from_slice(&rbuf[..4]);
        let pdu_length = u32::from_le_bytes(pdu_length);

        let mut pdu_flags = [0u8; 4];
        pdu_flags.copy_from_slice(&rbuf[4..8]);
        let pdu_flags = u32::from_le_bytes(pdu_flags);

        // Extend read string
        let rstr = String::from_utf8_lossy(&rbuf[PDU_HEADER_LENGTH..]);
        let rstr = rstr.trim_matches(char::from(0));
        read_string.push_str(rstr);
        tot_read += real_read - PDU_HEADER_LENGTH as u32;

        const CHANNEL_FLAG_ONLY: u32 = CHANNEL_FLAG_FIRST | CHANNEL_FLAG_LAST;
        match pdu_flags {
            CHANNEL_FLAG_ONLY /* 0x3 */ => {
                debug!("CHANNEL_FLAG_ONLY: one packet to read");
                break;
            }
            CHANNEL_FLAG_LAST /* 0x2 */ => {
                debug!("CHANNEL_FLAG_ONLY: last packet");
                specified_pdu_length = pdu_length;
                break;
            }
            CHANNEL_FLAG_FIRST /* 0x1 */ => {
                debug!("CHANNEL_FLAG_ONLY: first packet");
            }
            CHANNEL_FLAG_MIDDLE /* 0x0 */ => {
                debug!("CHANNEL_FLAG_MIDDLE: continuing...");
            }
            _ => {
                return Err(ws::core::Error::new(
                    ws::Win32::Foundation::E_FAIL,
                    format!("unsupported PDU flags: 0x{pdu_flags:x}"),
                ));
            }
        }
    }

    if specified_pdu_length != tot_read {
        return Err(ws::core::Error::new(
            ws::Win32::Foundation::E_FAIL,
            format!("inconsistent length: pdu_length = {specified_pdu_length} - read = {tot_read}"),
        ));
    }

    Ok(read_string)
}
