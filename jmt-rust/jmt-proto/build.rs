fn main() {
    prost_build::Config::new()
        .out_dir("src/")
        .compile_protos(
            &["proto/diagram.proto", "proto/commands.proto"],
            &["proto/"],
        )
        .expect("Failed to compile protobuf definitions");
}
