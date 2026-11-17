use protobuf::{Message, ProtobufEnum};

use super::test_codegen_options_pb::*;

#[test]
fn reflection_prefers_prost_names_over_legacy_names() {
    assert_eq!("ABC", Abc::new().descriptor().name());
    assert_eq!("A_B_C", ABC::new().descriptor().name());
    assert_eq!("XYZ", Xyz::enum_descriptor_static().name());
    assert_eq!("X_Y_Z", XYZ::enum_descriptor_static().name());
}

#[test]
fn reflection_lookup_supports_legacy_and_proto_names() {
    let file = file_descriptor_proto();
    let message = protobuf::descriptorx::find_message_by_rust_name(file, "option_test");
    assert_eq!("option_test", message.message.get_name());
    let message = protobuf::descriptorx::find_message_by_rust_name(file, ".ABC");
    assert_eq!("ABC", message.message.get_name());
    let en = protobuf::descriptorx::find_enum_by_rust_name(file, "option_test_inner_enum");
    assert_eq!("inner_enum", en.en.get_name());
    let en = protobuf::descriptorx::find_enum_by_rust_name(file, ".XYZ");
    assert_eq!("XYZ", en.en.get_name());
}

#[test]
fn file_options_enable_prost_names_and_pb_print() {
    let mut value = OptionTest::new();
    value.set_mode(OptionTestInnerEnum::SecondValue);
    value.set_text_value("example".to_owned());
    assert_eq!(
        protobuf::parse_from_bytes::<OptionTest>(&value.write_to_bytes().unwrap()).unwrap(),
        value
    );
    assert_eq!("option_test", value.descriptor().name());
    let mut output = String::new();
    protobuf::PbPrint::fmt(&value, "value", &mut output);
    assert!(!output.is_empty());
}
