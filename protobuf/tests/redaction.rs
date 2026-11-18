extern crate protobuf;

use protobuf::atomic_flags::{set_redact_level, RedactLevel};
use protobuf::well_known_types::{BytesValue, StringValue};
use protobuf::PbPrint;

#[test]
fn redacts_sensitive_values_in_all_printers() {
    let mut string = StringValue::new();
    string.set_value("secret".to_owned());
    let mut bytes = BytesValue::new();
    bytes.set_value(b"secret".to_vec());
    for &(level, expected) in &[
        (RedactLevel::Off, "secret"),
        (RedactLevel::On, "?"),
        (RedactLevel::Marker, "‹?›"),
    ] {
        set_redact_level(level);
        for message in [&string as &protobuf::Message, &bytes as &protobuf::Message].iter() {
            let output = protobuf::text_format::print_to_string(*message);
            assert!(output.contains(expected), "{}", output);
            if level != RedactLevel::Off {
                assert!(!output.contains("secret"));
            }
        }
        let mut output = String::new();
        PbPrint::fmt(&"secret".to_owned(), "s", &mut output);
        assert!(output.contains(expected), "{}", output);
        output.clear();
        PbPrint::fmt(&b"secret".to_vec(), "b", &mut output);
        if level != RedactLevel::Off {
            assert!(output.contains(expected), "{}", output);
            assert!(!output.contains("736563726574"));
            assert!(!output.contains("736563726574".to_uppercase().as_str()));
        }
    }
    set_redact_level(RedactLevel::Off);
}
