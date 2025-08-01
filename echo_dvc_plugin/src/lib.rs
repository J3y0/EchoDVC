mod class_factory;
mod echo_plugin;
mod logs;
mod registry;

use class_factory::EchoDVCClassFactory;
use echo_plugin::CLSID_ECHODVC_PLUGIN;
use log::{debug, error};
use registry::{com_register, com_unregister, rdp_register, rdp_unregister};
use windows::{self as ws, Win32::System::Com::IClassFactory};
use windows_core::Interface;

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn DllGetClassObject(
    rclsid: ws::core::Ref<ws::core::GUID>,
    riid: ws::core::Ref<ws::core::GUID>,
    ppv: ws::core::OutRef<IClassFactory>,
) -> ws::core::HRESULT {
    let _ = crate::logs::init_logs(log::LevelFilter::Debug, "plugin.log");

    let clsid = match rclsid.ok() {
        Ok(id) => *id,
        Err(e) => {
            error!("failed to get rclsid: {}", e);
            return ws::Win32::Foundation::E_INVALIDARG;
        }
    };

    let iid = match riid.ok() {
        Ok(id) => *id,
        Err(e) => {
            error!("failed to get riid: {}", e);
            return ws::Win32::Foundation::E_INVALIDARG;
        }
    };

    debug!("clsid: {clsid:?}");
    debug!("iid: {iid:?}");

    if iid != IClassFactory::IID {
        error!("invalid clsid: {iid:?}");
        return ws::Win32::Foundation::CLASS_E_CLASSNOTAVAILABLE;
    }

    let factory = EchoDVCClassFactory();

    if clsid != CLSID_ECHODVC_PLUGIN {
        error!("invalid clsid: {clsid:?}");
        ws::Win32::Foundation::CLASS_E_CLASSNOTAVAILABLE
    } else {
        let _ = ppv.write(Some(factory.into()));
        ws::Win32::Foundation::S_OK
    }
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn DllCanUnloadNow() -> ws::core::HRESULT {
    eprintln!("DllCanUnloadNow called");
    ws::Win32::Foundation::S_OK
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn DllRegisterServer() -> ws::core::HRESULT {
    let _ = unsafe { ws::Win32::System::Console::AllocConsole() };

    if let Err(e) = rdp_register() {
        eprintln!("RDP register error: {e}");
        return ws::Win32::System::Ole::SELFREG_E_CLASS;
    }

    if let Err(e) = com_register() {
        eprintln!("COM register error: {e}");
        return ws::Win32::System::Ole::SELFREG_E_CLASS;
    }

    eprintln!("ECHODVC plugin registered");
    ws::Win32::Foundation::S_OK
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn DllUnregisterServer() -> ws::core::HRESULT {
    let _ = unsafe { ws::Win32::System::Console::AllocConsole() };

    if let Err(e) = rdp_unregister() {
        eprintln!("RDP unregister error: {e}");
        return ws::Win32::System::Ole::SELFREG_E_CLASS;
    }

    if let Err(e) = com_unregister() {
        eprintln!("COM unregister error: {e}");
        return ws::Win32::System::Ole::SELFREG_E_CLASS;
    }

    eprintln!("ECHODVC plugin unregistered");

    ws::Win32::Foundation::S_OK
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn DllInstall(
    bInstall: bool,
    _pszCmdLine: ws::core::PCWSTR,
) -> ws::core::HRESULT {
    eprintln!("DllInstall called");

    match bInstall {
        true => DllRegisterServer(),
        false => DllUnregisterServer(),
    }
}
