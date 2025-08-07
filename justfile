# Build all
all: debug release

# Build client and server in debug mode
debug: server-debug client-debug

# Build client and server in release mode
release: server-release client-release

# Build server in debug mode
[working-directory: "echo_dvc_server"]
server-debug:
    @echo "Building server in debug mode"
    cargo build --target x86_64-pc-windows-gnu

# Build client in debug mode
[working-directory: "echo_dvc_plugin"]
client-debug:
    @echo "Building client in debug mode"
    cargo build --target i686-pc-windows-gnu
    cargo build --target x86_64-pc-windows-gnu

# Build server in release mode
[working-directory: "echo_dvc_server"]
server-release:
    @echo "Building server in release mode"
    cargo build --target x86_64-pc-windows-gnu --release

# Build client in release mode
[working-directory: "echo_dvc_plugin"]
client-release:
    @echo "Building client in release mode"
    cargo build --target i686-pc-windows-gnu --release
    cargo build --target x86_64-pc-windows-gnu --release

# Clean projects
clean: (_clean-path "echo_dvc_plugin") (_clean-path "echo_dvc_server")

_clean-path path:
    @echo "cleaning {{path}}..."
    cd {{path}} && cargo clean
