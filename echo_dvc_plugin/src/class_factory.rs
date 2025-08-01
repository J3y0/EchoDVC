use log::debug;
use std::ffi::c_void;
use std::ptr::null_mut;
use windows as ws;
use windows::Win32::System::Com::{IClassFactory, IClassFactory_Impl};
use windows::Win32::System::RemoteDesktop::IWTSPlugin;
use windows::core::implement;
use windows_core::Interface;

use crate::echo_plugin::EchoDvcPlugin;

#[implement(IClassFactory)]
pub struct EchoDVCClassFactory();

impl IClassFactory_Impl for EchoDVCClassFactory_Impl {
    fn CreateInstance(
        &self,
        outer: ws::core::Ref<'_, ws::core::IUnknown>,
        iid: *const ws::core::GUID,
        ppobject: *mut *mut c_void,
    ) -> Result<(), ws::core::Error> {
        let iid = unsafe { *iid };
        let ppobject = unsafe { &mut *ppobject };

        debug!("iid: {iid:?}");

        if outer.is_some() {
            return Err(ws::core::Error::from(
                ws::Win32::Foundation::CLASS_E_NOAGGREGATION,
            ));
        }

        *ppobject = null_mut();

        match iid {
            IWTSPlugin::IID => {
                debug!("IWTSPlugin request");
                let plugin: IWTSPlugin = EchoDvcPlugin().into();
                *ppobject = unsafe { std::mem::transmute::<IWTSPlugin, *mut c_void>(plugin) };
            }
            _ => return Err(ws::core::Error::from(ws::Win32::Foundation::E_NOINTERFACE)),
        }

        Ok(())
    }

    fn LockServer(&self, _lock: ws::core::BOOL) -> Result<(), ws::core::Error> {
        debug!("LockServer called");
        Ok(())
    }
}
