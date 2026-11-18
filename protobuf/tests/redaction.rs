extern crate protobuf;

use protobuf::atomic_flags::{set_redact_level, RedactLevel};
use protobuf::well_known_types::{BytesValue, StringValue};
use protobuf::PbPrint;

#[test]
fn applies_redaction_levels_to_all_printers() {
    let mut string = StringValue::new();
    string.set_value("key".to_owned());
    let mut bytes = BytesValue::new();
    bytes.set_value(b"key".to_vec());
    for &(level, text_value, string_value, bytes_value) in &[
        (RedactLevel::Off, "\"key\"", "\"key\"", "6B6579"),
        (RedactLevel::On, "\"?\"", "?", "?"),
        (RedactLevel::Marker, "\"‹6B6579›\"", "‹6B6579›", "‹6B6579›"),
    ] {
        set_redact_level(level);
        for message in [&string as &protobuf::Message, &bytes as &protobuf::Message].iter() {
            let expected = format!("value: {}", text_value);
            assert_eq!(protobuf::text_format::print_to_string(*message), expected);
            let mut output = String::new();
            protobuf::text_format::print_to(*message, &mut output);
            assert_eq!(output, expected);
            assert_eq!(format!("{:?}", message), expected);
            assert_eq!(format!("{:#?}", message), format!("{}\n", expected));
        }
        let mut output = String::new();
        PbPrint::fmt(&"key".to_owned(), "s", &mut output);
        assert_eq!(output, format!("s: {}", string_value));
        output.clear();
        PbPrint::fmt(&b"key".to_vec(), "b", &mut output);
        assert_eq!(output, format!("b: {}", bytes_value));

        #[cfg(feature = "bytes")]
        {
            output.clear();
            PbPrint::fmt(&protobuf::Chars::from("key"), "c", &mut output);
            assert_eq!(output, format!("c: {}", bytes_value));
        }
    }
    set_redact_level(RedactLevel::Off);
}
