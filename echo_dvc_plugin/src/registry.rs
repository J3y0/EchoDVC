use crate::echo_plugin::CLSID_ECHODVC_PLUGIN;
use std::env;

const RDP_ADDINS_PATH: &str = "Software\\Microsoft\\Terminal Server Client\\Default\\AddIns";

const PLUGIN_NAME: &str = env!("CARGO_CRATE_NAME");
const NAME_ENTRY: &str = "Name";
const THREADING_MODEL_ENTRY: &str = "ThreadingModel";

pub fn rdp_register() -> Result<(), String> {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);

    let (addins_key, _disp) = hkcu
        .create_subkey(RDP_ADDINS_PATH)
        .map_err(|e| format!("failed to create addins: {e}"))?;

    let (plugin, _disp) = addins_key
        .create_subkey(PLUGIN_NAME)
        .map_err(|e| format!("failed to create entry: {e}"))?;

    plugin
        .set_value(NAME_ENTRY, &format!("{{{CLSID_ECHODVC_PLUGIN:?}}}"))
        .map_err(|e| format!("failed to set name: {e}"))?;

    Ok(())
}

pub fn rdp_unregister() -> Result<(), String> {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let addins = hkcu
        .open_subkey_with_flags(RDP_ADDINS_PATH, winreg::enums::KEY_ALL_ACCESS)
        .map_err(|err| format!("failed to open rdp addins path: {err}"))?;
    addins
        .delete_subkey_all(PLUGIN_NAME)
        .map_err(|err| format!("failed to delete plugin entry: {err}"))?;

    Ok(())
}

pub fn com_register() -> Result<(), String> {
    let hkcr = winreg::RegKey::predef(winreg::enums::HKEY_CLASSES_ROOT);

    let (inproc, _disp) = hkcr
        .create_subkey(format!(
            "CLSID\\{{{CLSID_ECHODVC_PLUGIN:?}}}\\InprocServer32"
        ))
        .map_err(|err| format!("failed to open clsid path: {err}"))?;

    let mut dll = env::current_dir().unwrap();
    #[cfg(target_arch = "x86")]
    dll.push(format!("{PLUGIN_NAME}_32.dll"));
    #[cfg(target_arch = "x86_64")]
    dll.push(format!("{PLUGIN_NAME}.dll"));
    let dll_path = dll.to_str().unwrap();

    inproc
        .set_value("", &dll_path)
        .map_err(|err| format!("failed to set default inprocserver32 value: {err}"))?;
    inproc
        .set_value(THREADING_MODEL_ENTRY, &"Free")
        .map_err(|err| format!("failed to set threading model value: {err}"))?;
    Ok(())
}

pub fn com_unregister() -> Result<(), String> {
    let hkcr = winreg::RegKey::predef(winreg::enums::HKEY_CLASSES_ROOT);
    let (clsid, _disp) = hkcr
        .create_subkey_with_flags("CLSID", winreg::enums::KEY_ALL_ACCESS)
        .map_err(|err| format!("failed to open clsid path: {err}"))?;

    let plugin_clsid_path = format!("{{{CLSID_ECHODVC_PLUGIN:?}}}");
    clsid
        .delete_subkey_all(plugin_clsid_path)
        .map_err(|err| format!("failed to delete plugin clsid: {err}"))?;

    Ok(())
}
