extern crate heck;
extern crate protobuf;

use std::collections::hash_map::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;

use protobuf::compiler_plugin;
use protobuf::descriptor::*;
use protobuf::descriptorx::*;
use protobuf::Message;

mod customize;
mod enums;
mod extensions;
mod field;
mod file_and_mod;
pub mod float;
mod ident;
mod inside;
mod message;
mod oneof;
mod rust_name;
mod rust_types_values;
mod serde;
mod well_known_types;

use customize::customize_from_rustproto_for_file;
pub use customize::Customize;

pub mod code_writer;

use self::code_writer::CodeWriter;
use self::enums::*;
use self::extensions::*;
use self::message::*;
use inside::protobuf_crate_path;

fn escape_byte(s: &mut String, b: u8) {
    if b == b'\n' {
        write!(s, "\\n").unwrap();
    } else if b == b'\r' {
        write!(s, "\\r").unwrap();
    } else if b == b'\t' {
        write!(s, "\\t").unwrap();
    } else if b == b'\\' || b == b'"' {
        write!(s, "\\{}", b as char).unwrap();
    } else if b == b'\0' {
        write!(s, "\\0").unwrap();
    // ASCII printable except space
    } else if b > 0x20 && b < 0x7f {
        write!(s, "{}", b as char).unwrap();
    } else {
        write!(s, "\\x{:02x}", b).unwrap();
    }
}

fn write_file_descriptor_data(
    file: &FileDescriptorProto,
    customize: &Customize,
    w: &mut CodeWriter,
) {
    let fdp_bytes = file.write_to_bytes().unwrap();
    w.write_line("static file_descriptor_proto_data: &'static [u8] = b\"\\");
    w.indented(|w| {
        const MAX_LINE_LEN: usize = 72;

        let mut s = String::new();
        for &b in &fdp_bytes {
            let prev_len = s.len();
            escape_byte(&mut s, b);
            let truncate = s.len() > MAX_LINE_LEN;
            if truncate {
                s.truncate(prev_len);
            }
            if truncate || s.len() == MAX_LINE_LEN {
                write!(s, "\\").unwrap();
                w.write_line(&s);
                s.clear();
            }
            if truncate {
                escape_byte(&mut s, b);
            }
        }
        if !s.is_empty() {
            write!(s, "\\").unwrap();
            w.write_line(&s);
            s.clear();
        }
    });
    w.write_line("\";");
    w.write_line("");
    w.lazy_static_protobuf_path(
        "file_descriptor_proto_lazy",
        &format!(
            "{}::descriptor::FileDescriptorProto",
            protobuf_crate_path(customize)
        ),
        protobuf_crate_path(customize),
    );
    w.write_line("");
    w.def_fn(
        "parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto",
        |w| {
            w.write_line("::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()");
        },
    );
    w.write_line("");
    w.pub_fn(
        "file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto",
        |w| {
            w.unsafe_expr(|w| {
                w.block("file_descriptor_proto_lazy.get(|| {", "})", |w| {
                    w.write_line("parse_descriptor_proto()");
                });
            });
        },
    );
}

fn gen_file(
    file: &FileDescriptorProto,
    _files_map: &HashMap<&str, &FileDescriptorProto>,
    root_scope: &RootScope,
    customize: &Customize,
) -> Option<compiler_plugin::GenResult> {
    // TODO: use it
    let mut customize = customize.clone();
    // options specified in invocation have precedence over options specified in file
    customize.update_with(&customize_from_rustproto_for_file(file.get_options()));

    let scope = FileScope {
        file_descriptor: file,
        prost_style: customize.prost_style.unwrap_or(true),
    }
    .to_scope();
    let lite_runtime = customize.lite_runtime.unwrap_or_else(|| {
        file.get_options().get_optimize_for() == FileOptions_OptimizeMode::LITE_RUNTIME
    });

    let mut v = Vec::new();

    {
        let mut w = CodeWriter::new(&mut v);

        w.write_generated_by("rust-protobuf", env!("CARGO_PKG_VERSION"));
        w.write_line(&format!("//! Generated file from `{}`", file.get_name()));

        w.write_line("");
        w.write_line("use protobuf::Message as Message_imported_for_functions;");
        w.write_line("use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;");
        if customize.inside_protobuf != Some(true) {
            w.write_line("");
            w.write_line("/// Generated files are compatible only with the same version");
            w.write_line("/// of protobuf runtime.");
            w.write_line(&format!(
                "const _PROTOBUF_VERSION_CHECK: () = {}::{};",
                protobuf_crate_path(&customize),
                protobuf::VERSION_IDENT
            ));
        }

        for message in &scope.get_messages() {
            // ignore map entries, because they are not used in map fields
            if message.map_entry().is_none() {
                w.write_line("");
                MessageGen::new(message, &root_scope, &customize).write(&mut w);
            }
        }
        for enum_type in &scope.get_enums() {
            w.write_line("");
            EnumGen::new(enum_type, file, &customize, root_scope).write(&mut w);
        }

        write_extensions(file, &root_scope, &mut w, &customize);

        if !lite_runtime {
            w.write_line("");
            write_file_descriptor_data(file, &customize, &mut w);
        }
    }

    Some(compiler_plugin::GenResult {
        name: format!("{}.rs", proto_path_to_rust_mod(file.get_name())),
        content: v,
    })
}

// This function is also used externally by cargo plugin
// https://github.com/plietar/rust-protobuf-build
// So be careful changing its signature.
pub fn gen(
    file_descriptors: &[FileDescriptorProto],
    files_to_generate: &[String],
    customize: &Customize,
) -> Vec<compiler_plugin::GenResult> {
    let root_scope = RootScope {
        file_descriptors: file_descriptors,
        prost_style: customize.prost_style.unwrap_or(true),
    };

    let mut results: Vec<compiler_plugin::GenResult> = Vec::new();
    let files_map: HashMap<&str, &FileDescriptorProto> =
        file_descriptors.iter().map(|f| (f.get_name(), f)).collect();

    let all_file_names: Vec<&str> = file_descriptors.iter().map(|f| f.get_name()).collect();

    for file_name in files_to_generate {
        let file = files_map.get(&file_name[..]).expect(&format!(
            "file not found in file descriptors: {:?}, files: {:?}",
            file_name, all_file_names
        ));
        results.extend(gen_file(file, &files_map, &root_scope, customize));
    }
    results
}

pub fn gen_and_write(
    file_descriptors: &[FileDescriptorProto],
    files_to_generate: &[String],
    out_dir: &Path,
    customize: &Customize,
) -> io::Result<()> {
    let results = gen(file_descriptors, files_to_generate, customize);

    for r in &results {
        let mut file_path = out_dir.to_owned();
        file_path.push(&r.name);
        let mut file_writer = File::create(&file_path)?;
        file_writer.write_all(&r.content)?;
        file_writer.flush()?;
    }

    Ok(())
}

pub fn protoc_gen_rust_main() {
    compiler_plugin::plugin_main_2(|r| {
        let customize = Customize::parse_from_parameter(r.parameter).expect("parse options");
        gen(r.file_descriptors, r.files_to_generate, &customize)
    });
}

#[test]
fn generated_any_methods_support_rust_2015() {
    let mut file = FileDescriptorProto::new();
    file.set_name("test.proto".to_owned());
    let mut message = protobuf::descriptor::DescriptorProto::new();
    message.set_name("Test".to_owned());
    file.mut_message_type().push(message);

    let generated = gen(&[file], &["test.proto".to_owned()], &Customize::default());
    let code = ::std::str::from_utf8(&generated[0].content).unwrap();
    assert!(code.contains("fn as_any(&self) -> &dyn (::std::any::Any)"));
    assert!(code.contains("self as &dyn (::std::any::Any)"));
    assert!(code.contains("fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any)"));
    assert!(code.contains("self as &mut dyn (::std::any::Any)"));
    assert!(code.contains("Box<dyn (::std::any::Any)>"));
}

#[test]
fn naming_and_pb_print_options_are_independent() {
    let mut file = FileDescriptorProto::new();
    file.set_name("options.proto".to_owned());
    let mut message = protobuf::descriptor::DescriptorProto::new();
    message.set_name("example_message".to_owned());
    file.mut_message_type().push(message);

    for &prost_style in &[false, true] {
        for &generate_pb_print in &[false, true] {
            let customize = Customize {
                prost_style: Some(prost_style),
                generate_pb_print: Some(generate_pb_print),
                ..Default::default()
            };
            let generated = gen(&[file.clone()], &["options.proto".to_owned()], &customize);
            let code = ::std::str::from_utf8(&generated[0].content).unwrap();
            let name = if prost_style {
                "ExampleMessage"
            } else {
                "example_message"
            };
            assert!(code.contains(&format!("pub struct {}", name)));
            assert_eq!(
                code.contains(&format!("impl ::protobuf::PbPrint for {}", name)),
                generate_pb_print
            );
            assert_eq!(
                code.contains("::protobuf::text_format::fmt(self, f)"),
                !generate_pb_print
            );
        }
    }
}

#[test]
fn file_options_and_plugin_parameters_are_parsed() {
    let extensions = protobuf::rustproto::file_descriptor_proto().get_extension();
    assert!(extensions.iter().any(|e| e.get_name() == "prost_style_all" && e.get_number() == 18000));
    assert!(extensions.iter().any(|e| e.get_name() == "generate_pb_print_all" && e.get_number() == 18001));
    let customize =
        Customize::parse_from_parameter("prost_style=false generate_pb_print=false").unwrap();
    assert_eq!(customize.prost_style, Some(false));
    assert_eq!(customize.generate_pb_print, Some(false));

    let mut file = FileDescriptorProto::new();
    file.set_name("options.proto".to_owned());
    file.mut_options().mut_unknown_fields().add_varint(18000, 0);
    file.mut_options().mut_unknown_fields().add_varint(18001, 0);
    let mut message = protobuf::descriptor::DescriptorProto::new();
    message.set_name("example_message".to_owned());
    file.mut_message_type().push(message);
    let generated = gen(
        &[file],
        &["options.proto".to_owned()],
        &Customize::default(),
    );
    let code = ::std::str::from_utf8(&generated[0].content).unwrap();
    assert!(code.contains("pub struct example_message"));
    assert!(!code.contains("impl ::protobuf::PbPrint for example_message"));
}
