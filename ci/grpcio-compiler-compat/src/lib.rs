// Sharing these types verifies that grpcio-compiler uses this checkout's
// protobuf crate rather than a separate crates.io version.
pub fn generate(
    files: &[protobuf::descriptor::FileDescriptorProto],
    names: &[String],
) -> Vec<protobuf::compiler_plugin::GenResult> {
    grpcio_compiler::codegen::gen(files, names)
}

pub fn generate_messages(
    files: &[protobuf::descriptor::FileDescriptorProto],
    names: &[String],
) -> Vec<protobuf::compiler_plugin::GenResult> {
    protobuf_codegen::gen(files, names, &protobuf_codegen::Customize::default())
}
