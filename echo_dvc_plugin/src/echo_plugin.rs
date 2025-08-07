use log::{debug, error, info};

use windows::Win32::System::RemoteDesktop::{
    IWTSListenerCallback, IWTSListenerCallback_Impl, IWTSPlugin, IWTSPlugin_Impl,
    IWTSVirtualChannel, IWTSVirtualChannelCallback, IWTSVirtualChannelCallback_Impl,
    IWTSVirtualChannelManager,
};
use windows::{
    self as ws,
    core::{ComObjectInterface, Error, GUID, PCSTR, implement},
};

pub const CLSID_ECHODVC_PLUGIN: GUID = GUID::from_u128(0xF5234ABFAC884D6EAA8D490DF08F194D);
const DVC_NAME: &str = "ECHOCHN";

#[implement(IWTSPlugin, IWTSListenerCallback)]
pub struct EchoDvcPlugin();

impl IWTSPlugin_Impl for EchoDvcPlugin_Impl {
    fn Initialize(
        &self,
        p_channel_manager: ws::core::Ref<IWTSVirtualChannelManager>,
    ) -> Result<(), ws::core::Error> {
        info!("CALLED initialized");
        debug!("DVC name is {DVC_NAME:?}");

        match p_channel_manager.as_ref() {
            None => {
                return Err(Error::new(
                    ws::Win32::Foundation::E_INVALIDARG,
                    "channel manager is null",
                ));
            }
            Some(channel_manager) => {
                debug!("channel_manager ok");

                let flags = 0;
                let _ = unsafe {
                    channel_manager.CreateListener(
                        PCSTR(format!("{DVC_NAME}\0").as_ptr()),
                        flags,
                        self.as_interface_ref(),
                    )?
                };

                info!("listener created for channel: {DVC_NAME}");
            }
        }

        Ok(())
    }

    fn Connected(&self) -> Result<(), ws::core::Error> {
        info!("client connected");
        Ok(())
    }

    fn Disconnected(&self, disconnect_code: u32) -> Result<(), ws::core::Error> {
        info!("client disconnected with: {disconnect_code}");
        Ok(())
    }

    fn Terminated(&self) -> Result<(), ws::core::Error> {
        info!("client terminated");
        Ok(())
    }
}

impl IWTSListenerCallback_Impl for EchoDvcPlugin_Impl {
    fn OnNewChannelConnection(
        &self,
        channel_ref: ws::core::Ref<'_, IWTSVirtualChannel>,
        _data: &ws::core::BSTR,
        paccept: *mut ws::core::BOOL,
        p_callback: ws::core::OutRef<'_, IWTSVirtualChannelCallback>,
    ) -> Result<(), windows_core::Error> {
        info!("CALLED OnNewChannelConnection");

        let channel = channel_ref
            .ok()
            .inspect_err(|err| error!("failed to get channel ref: {err}"))?;

        let channel_callback: IWTSVirtualChannelCallback =
            EchoDvcChannelCallback::new(channel).into();

        p_callback
            .write(Some(channel_callback))
            .inspect_err(|err| error!("failed to write virtual channel callback: {err}"))?;

        debug!("VirtualChannelCallback ok");

        if let Some(accept) = unsafe { paccept.as_mut() } {
            *accept = ws::Win32::Foundation::TRUE;
        }

        Ok(())
    }
}

#[implement(IWTSVirtualChannelCallback)]
pub struct EchoDvcChannelCallback {
    channel: ws::core::AgileReference<IWTSVirtualChannel>,
}

impl EchoDvcChannelCallback {
    fn new(channel: &IWTSVirtualChannel) -> Self {
        Self {
            channel: ws::core::AgileReference::new(channel).unwrap(),
        }
    }
}

impl IWTSVirtualChannelCallback_Impl for EchoDvcChannelCallback_Impl {
    fn OnDataReceived(&self, size: u32, buffer: *const u8) -> Result<(), ws::core::Error> {
        info!("CALLED OnDataReceived");

        let received_buffer = unsafe { std::slice::from_raw_parts(buffer, size as usize) };
        debug!(
            "received: {} ({:?})",
            String::from_utf8_lossy(received_buffer),
            received_buffer
        );

        let to_send = received_buffer;
        unsafe { self.channel.resolve().unwrap().Write(to_send, None) }
            .inspect_err(|err| error!("failed to write to channel: {err}"))?;

        debug!("sent: {} ({:?})", String::from_utf8_lossy(to_send), to_send);

        Ok(())
    }

    fn OnClose(&self) -> Result<(), ws::core::Error> {
        info!("CALLED OnClose");
        Ok(())
    }
}
