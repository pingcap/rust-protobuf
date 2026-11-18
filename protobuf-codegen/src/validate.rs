//! Validate descriptor names before using them as Rust identifiers or paths.

use protobuf::descriptor::{
    DescriptorProto, EnumDescriptorProto, FieldDescriptorProto, FileDescriptorProto,
};

fn ident(name: &str) -> bool {
    let mut chars = name.bytes();
    match chars.next() {
        Some(b'a'..=b'z') | Some(b'A'..=b'Z') | Some(b'_') => (),
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == b'_')
}

fn name(kind: &str, value: &str) -> Result<(), String> {
    if ident(value) {
        Ok(())
    } else {
        Err(format!("invalid {} name: {:?}", kind, value))
    }
}

fn path(kind: &str, value: &str) -> Result<(), String> {
    let value = value.strip_prefix('.').unwrap_or(value);
    if !value.is_empty() && value.split('.').all(ident) {
        Ok(())
    } else {
        Err(format!("invalid {}: {:?}", kind, value))
    }
}

fn field(f: &FieldDescriptorProto) -> Result<(), String> {
    name("field", f.get_name())?;
    if f.has_type_name() {
        path("field type", f.get_type_name())?;
    }
    if f.has_extendee() {
        path("extendee", f.get_extendee())?;
    }
    Ok(())
}

fn enumeration(e: &EnumDescriptorProto) -> Result<(), String> {
    name("enum", e.get_name())?;
    for v in e.get_value() {
        name("enum value", v.get_name())?;
    }
    Ok(())
}

fn message(m: &DescriptorProto) -> Result<(), String> {
    name("message", m.get_name())?;
    for f in m.get_field().iter().chain(m.get_extension()) {
        field(f)?;
    }
    for o in m.get_oneof_decl() {
        name("oneof", o.get_name())?;
    }
    for m in m.get_nested_type() {
        message(m)?;
    }
    for e in m.get_enum_type() {
        enumeration(e)?;
    }
    Ok(())
}

pub fn file(f: &FileDescriptorProto) -> Result<(), String> {
    if !f.get_package().is_empty() {
        path("package", f.get_package())?;
    }
    for m in f.get_message_type() {
        message(m)?;
    }
    for e in f.get_enum_type() {
        enumeration(e)?;
    }
    for field in f.get_extension() {
        self::field(field)?;
    }
    for service in f.get_service() {
        name("service", service.get_name())?;
        for method in service.get_method() {
            name("method", method.get_name())?;
            if method.has_input_type() {
                path("input type", method.get_input_type())?;
            }
            if method.has_output_type() {
                path("output type", method.get_output_type())?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_descriptor_injection_in_names_and_paths() {
        let mut file = FileDescriptorProto::new();
        let mut message = DescriptorProto::new();
        message.set_name("Test".to_owned());
        let mut field = FieldDescriptorProto::new();
        field.set_name("valid".to_owned());
        field.set_type_name(".pkg.Type\"); injected!(".to_owned());
        message.mut_field().push(field);
        file.mut_message_type().push(message);
        assert!(self::file(&file).is_err());

        file.mut_message_type()[0].mut_field()[0].clear_type_name();
        file.mut_message_type()[0].mut_field()[0].set_name("x\"); injected!(".to_owned());
        assert!(self::file(&file).is_err());

        file.mut_message_type()[0].mut_field()[0].set_name("valid".to_owned());
        let mut oneof = ::protobuf::descriptor::OneofDescriptorProto::new();
        oneof.set_name("x { injected!(); }".to_owned());
        file.mut_message_type()[0].mut_oneof_decl().push(oneof);
        assert!(self::file(&file).is_err());
    }
}
