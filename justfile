# Build client and server
build: server client

# Build server
[working-directory: "echo_dvc_server"]
server:
    @echo "Building server"
    cargo build --target x86_64-pc-windows-gnu
    cp ./target/x86_64-pc-windows-gnu/debug/echo_dvc_server.exe ~/Documents/vm/shared/echo_dvc_server.exe

# Build client
[working-directory: "echo_dvc_plugin"]
client:
    @echo "Building client"
    cargo build --target i686-pc-windows-gnu
    cargo build --target x86_64-pc-windows-gnu
    cp ./target/i686-pc-windows-gnu/debug/echo_dvc_plugin.dll ~/Documents/vm/shared/echo_dvc_plugin_32.dll
    cp ./target/x86_64-pc-windows-gnu/debug/echo_dvc_plugin.dll ~/Documents/vm/shared/echo_dvc_plugin.dll

# Clean projects
clean: (_clean-path "echo_dvc_plugin") (_clean-path "echo_dvc_server")

_clean-path path:
    @echo "cleaning {{path}}..."
    cd {{path}} && cargo clean
