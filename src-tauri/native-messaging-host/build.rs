use std::fs;
use std::path::Path;

fn main() {
    generate_code_from_proto(Path::new("src/proto"));
}

fn generate_code_from_proto(proto_out_dir: &Path) {
    let proto_files = [
        "../../proto/native_messaging/common.proto",
        "../../proto/native_messaging/status.proto",
        "../../proto/native_messaging/sync.proto",
        "../../proto/native_messaging/packs.proto",
    ];

    for proto_file in &proto_files {
        println!("cargo:rerun-if-changed={}", proto_file);
    }

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let descriptor_path = proto_out_dir.join("descriptor.bin");
    let proto_out_dir = proto_out_dir.join("generated");

    fs::create_dir_all(&proto_out_dir).unwrap();

    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor_path)
        .compile_well_known_types()
        .extern_path(".google.protobuf", "::pbjson_types")
        .compile_protos(&proto_files, &["../../proto"]).unwrap();

    let descriptor_set = std::fs::read(descriptor_path).unwrap();
    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_set).unwrap()
        .build(&[".launcherg"]).unwrap();

    let patterns = [
        "launcherg.common.rs",
        "launcherg.common.serde.rs",
        "launcherg.sync.rs",
        "launcherg.sync.serde.rs",
        "launcherg.status.rs",
        "launcherg.status.serde.rs",
        "launcherg.packs.rs",
        "launcherg.packs.serde.rs",
    ];

    for pattern in &patterns {
        let src_path = Path::new(&out_dir).join(pattern);
        let dst_path = proto_out_dir.join(pattern);
        if src_path.exists() {
            fs::copy(&src_path, &dst_path).unwrap();
        }
    }
}
