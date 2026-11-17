extern crate protobuf;

use protobuf::error::WireError;
use protobuf::well_known_types::{ListValue, Struct, Value};
use protobuf::wire_format::WireTypeLengthDelimited;
use protobuf::{
    CodedInputStream, Message, ProtobufError, ProtobufResult, RepeatedField, SingularPtrField,
};

fn assert_limit<T>(result: ProtobufResult<T>) {
    let error = result.err();
    assert!(
        matches!(
            error,
            Some(ProtobufError::WireError(WireError::OverRecursionLimit))
        ),
        "expected recursion limit error, got {:?}",
        error
    );
}

// Each layer is Struct.fields -> map entry value -> Value.struct_value -> Struct.
fn nested_struct(depth: usize) -> Vec<u8> {
    let mut message = Struct::new();
    for _ in 0..depth {
        let mut value = Value::new();
        value.set_struct_value(message);
        message = Struct::new();
        message.mut_fields().insert(String::new(), value);
    }
    message.write_to_bytes().unwrap()
}

// Each layer is Value.list_value -> ListValue.values -> Value.
fn nested_list(depth: usize) -> Vec<u8> {
    let mut value = Value::new();
    for _ in 0..depth {
        let mut list = ListValue::new();
        list.mut_values().push(value);
        value = Value::new();
        value.set_list_value(list);
    }
    value.write_to_bytes().unwrap()
}

fn check_depth<M: Message>(bytes: &[u8], depth: u32) {
    let mut is = CodedInputStream::from_bytes(bytes);
    is.set_recursion_limit(depth - 1);
    assert_limit(M::new().merge_from(&mut is));

    let mut is = CodedInputStream::from_bytes(bytes);
    is.set_recursion_limit(depth);
    M::new().merge_from(&mut is).unwrap();
    assert!(is.eof().unwrap());
}

#[test]
fn map_and_oneof_messages_obey_recursion_limit() {
    check_depth::<Struct>(&nested_struct(10), 20);
    check_depth::<Value>(&nested_list(10), 20);
    assert_limit(protobuf::parse_from_bytes::<Struct>(&nested_struct(60)));
    assert_limit(protobuf::parse_from_bytes::<Value>(&nested_list(60)));
}

#[test]
fn direct_message_reads_obey_recursion_limit() {
    let mut is = CodedInputStream::from_bytes(&[0]);
    is.set_recursion_limit(0);
    assert_limit(is.read_message::<Value>());

    let mut is = CodedInputStream::from_bytes(&[0]);
    is.set_recursion_limit(0);
    assert_limit(is.merge_message(&mut Value::new()));

    let mut is = CodedInputStream::from_bytes(&[0]);
    is.set_recursion_limit(1);
    is.read_message::<Value>().unwrap();
}

#[test]
fn singular_and_repeated_messages_count_once() {
    for limit in 0..=1 {
        let mut is = CodedInputStream::from_bytes(&[0]);
        is.set_recursion_limit(limit);
        let mut target = SingularPtrField::<Value>::none();
        let result =
            protobuf::rt::read_singular_message_into(WireTypeLengthDelimited, &mut is, &mut target);
        if limit == 0 {
            assert_limit(result);
        } else {
            result.unwrap();
        }

        let mut is = CodedInputStream::from_bytes(&[0]);
        is.set_recursion_limit(limit);
        let mut target = RepeatedField::<Value>::new();
        let result =
            protobuf::rt::read_repeated_message_into(WireTypeLengthDelimited, &mut is, &mut target);
        if limit == 0 {
            assert_limit(result);
        } else {
            result.unwrap();
        }
    }
}

#[test]
fn failed_message_read_restores_depth_and_byte_limit() {
    // The first message contains an invalid tag; the second is empty.
    let mut is = CodedInputStream::from_bytes(&[1, 0, 0]);
    is.set_recursion_limit(1);
    assert!(is.read_message::<Value>().is_err());
    is.read_message::<Value>().unwrap();
}
