extern crate protobuf;

use protobuf::descriptor::FileDescriptorProto;
use protobuf::descriptorx::{FileScope, RootScope, Scope};

#[test]
fn public_scope_struct_literals_remain_compatible() {
    let file = FileDescriptorProto::new();
    let files = vec![file.clone()];
    let _root = RootScope {
        file_descriptors: &files,
    };
    let file_scope = FileScope {
        file_descriptor: &file,
    };
    let _scope = Scope {
        file_scope,
        path: Vec::new(),
    };
}
