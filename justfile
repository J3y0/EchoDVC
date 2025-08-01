# Build client
build: client

# Build client
[working-directory: "echo_dvc_plugin"]
client:
    @echo "Building client"
    cargo build --target i686-pc-windows-gnu
    cargo build --target x86_64-pc-windows-gnu
    cp ./target/i686-pc-windows-gnu/debug/echo_dvc_plugin.dll ~/Documents/vm/shared/echo_dvc_plugin_32.dll
    cp ./target/x86_64-pc-windows-gnu/debug/echo_dvc_plugin.dll ~/Documents/vm/shared/echo_dvc_plugin.dll
