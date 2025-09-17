use std::env;
use std::path::PathBuf;

fn main() {
    // Tell Cargo to rerun build.rs if protobuf files change
    println!("cargo:rerun-if-changed=protowire/message.proto");
    println!("cargo:rerun-if-changed=protowire/rpc.proto");

    // Set output directory
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Configure tonic-build
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("protowire_descriptor.bin"))
        .compile(
            &["protowire/message.proto", "protowire/rpc.proto"],
            &["protowire"],
        )
        .unwrap();

    println!("cargo:warning=Protobuf code generated successfully");
}
