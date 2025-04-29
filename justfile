build-daemon:
    cargo build --package bedrockd

build-linux:
    cargo zigbuild --target x86_64-unknown-linux-gnu.2.17
