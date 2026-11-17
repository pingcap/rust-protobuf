# protobuf-codegen

This crate contains protobuf code generator and a `protoc-gen-rust` `protoc` plugin.

## protoc-gen-rust

`protoc-gen-rust` implements standard protobuf `protoc` plugin conventions.

Probably you do not want to use it directly in Rust environment, there are easier to use alternatives:

* [protoc-rust crate](https://github.com/stepancheg/rust-protobuf/tree/master/protoc-rust)
  which can be invoked programmatically from `build.rs` of your project
  which requires only `protoc` in `$PATH` but not `protoc-gen-rust`.
* [protobuf-codegen-pure crate](https://github.com/stepancheg/rust-protobuf/tree/master/protobuf-codegen-pure)
  which behaves like protoc-rust, but does not depend on `protoc` binary

## But if you really want to use that plugin, here's the instruction

(Note `protoc` can be invoked programmatically with
[protoc crate](https://github.com/stepancheg/rust-protobuf/tree/master/protoc/))

0) Install protobuf for `protoc` binary.

On OS X [Homebrew](https://github.com/Homebrew/brew) can be used:

```
brew install protobuf
```

On Ubuntu, `protobuf-compiler` package can be installed:

```
apt-get install protobuf-compiler
```

Protobuf is needed only for code generation, `rust-protobuf` runtime
does not use `protobuf` library.

1) Install `protoc-gen-rust` program (which is `protoc` plugin)

It can be installed either from source or with `cargo install protobuf` command.

2) Add `protoc-gen-rust` to $PATH

If you installed it with cargo, it should be

```
PATH="$HOME/.cargo/bin:$PATH"
```

3) Generate .rs files:

```
protoc --rust_out . foo.proto
```

This will generate .rs files in current directory.

## Naming and Debug options

This fork defaults to Prost-style identifiers and generates `PbPrint` and
redacting `Debug` implementations. To use legacy rust-protobuf names and
`Debug` behavior, set `prost_style` and `generate_pb_print` to `Some(false)`
in `Customize`, or set the file-level options in a `.proto` file:

```proto
import "rustproto.proto";
option (rustproto.prost_style_all) = false;
option (rustproto.generate_pb_print_all) = false;
```

The options are independent. Disabling `generate_pb_print` removes the
generated `PbPrint` implementation and its redaction behavior. Messages
generated with `PbPrint` enabled require `PbPrint` implementations for their
message and enum field types, including imported types.
