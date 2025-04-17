use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("rcon.bin"))
        .compile_protos(&["rcon.proto"], &["../proto"])
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));

    Ok(())
    // tonic_build::compile_protos("../proto/rcon.proto")
    //     .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}