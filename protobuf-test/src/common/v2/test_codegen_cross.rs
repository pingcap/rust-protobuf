use super::test_codegen_cross_pb::CrossFile;
use super::test_codegen_legacy_pb::Foo_Bar;

#[test]
fn prost_style_file_uses_legacy_import_name() {
    let mut cross = CrossFile::new();
    cross.set_legacy_message(Foo_Bar::new());
    assert!(cross.has_legacy_message());
}
