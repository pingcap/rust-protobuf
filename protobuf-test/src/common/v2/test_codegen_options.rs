use protobuf::Message;

use super::test_codegen_options_pb::*;

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
