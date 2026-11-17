use protobuf::Message;
use super::test_codegen_legacy_pb::{FooBar, Foo_Bar};

#[test]
fn reflection_distinguishes_legacy_names() {
    assert_eq!("FooBar", FooBar::new().descriptor().name());
    assert_eq!("Foo_Bar", Foo_Bar::new().descriptor().name());
}
