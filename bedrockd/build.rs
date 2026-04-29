use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Safety: build scripts run single-threaded, so setting env vars is safe here.
    unsafe { env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap()) };

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("rcon.bin"))
        .compile_protos(&["rcon/v1/rcon.proto"], &["../proto"])
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));

    Ok(())
    // tonic_build::compile_protos("../proto/rcon.proto")
    //     .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
