fn main() {
    println!("cargo:rerun-if-changed=../../bindings/proto/revault_bindings.proto");
    prost_build::Config::new()
        .compile_protos(
            &["../../bindings/proto/revault_bindings.proto"],
            &["../../bindings/proto"],
        )
        .expect("compile binding protobuf schema");
}
