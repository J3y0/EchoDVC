# Echo DVC

Echo DVC is a simple yet functional Remote Desktop Protocol (RDP) plugin implemented
using the **Dynamic Virtual Channel (DVC) API**. It currently supports *Microsoft RDP* and *Citrix*.

The goal of this project is to provide a **template for building RDP DVC plugins in
Rust**, covering both client and server sides. While the echo functionality is
minimal, it serves as a useful and lightweight implementation to validate DVC
communication.

Under the hood, the client plugin is implemented as a **COM object**. Because
it uses COM technology, the plugin is only available on Windows.

>⚙️ This project is ideal for developers looking to build and experiment with custom DVC plugins in Rust.

## ✨ Features

- RDP Dynamic Virtual Channel support
- Written in Rust
- COM-based client plugin for Windows
- Cross-compatibility with:
  - Citrix (Windows x86) — use x86 version
  - MSTSC (Microsoft Remote Desktop Connection) — use x64 version
- Simple echo server for basic data exchange testing

## Installation

You can find in the [Releases](https://github.com/J3y0/EchoDVC/releases) section the different binaries:

- DLL for the client
- executable for the server

Both have a 32 and 64 bits version depending on your target.

Use the 32bits version for *Citrix* and the 64bits for *Windows*.

### Client side (Windows)

The client-side DLL must be registered via the Windows registry. For this purpose,
the DLL exports DllRegisterServer and DllUnregisterServer, making it compatible
with regsvr32.exe.

To register the plugin:

```powershell
# 64bits registration
regsvr32.exe C:\Path\to\echo_dvc_plugin.dll

# 32bits registration
regsvr32.exe C:\Path\to\echo_dvc_plugin_32.dll
```

To unregister the plugin:

```powershell
# 64bits
regsvr32.exe /u C:\Path\to\echo_dvc_plugin.dll

# 32bits
regsvr32.exe /u C:\Path\to\echo_dvc_plugin_32.dll
```

### Server side

The server is a standalone executable that can be run on the remote machine once
a remote session has been established.

## Usage

Once the plugin is registered, connect to your favorite remote instance. If the
plugin has been correctly installed, `Remote Desktop Connection` client should
load it successfully.

From there, run the server binary. You should see the following prompt:

```
Usage:
- "write XXXX" or "put XXXX" to write to the DVC
- "quit" or "exit" to leave this interface

echo_dvc> 
```

>💡 If you changed the plugin DVC name, you **do not** need to rebuild the server binary as you can override the DVC name on the commandline. Please see the help for further information on the usage: `.\echo_dvc_server.exe --help`

## ✅ Compatibility

| Environment | Architecture | Compatible |
| -------- | ------ | -----|
| Microsoft RDP (MSTSC) | x64 | ✅ |
| Citrix Receiver | x86 | ✅ |

Make sure to build or use the correct DLL architecture according to your client.
