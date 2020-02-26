use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write as _;
use std::io::Read;
use std::io::Write;

#[cfg(feature = "bytes")]
use bytes::Bytes;

use clear::Clear;
use error::ProtobufError;
use error::ProtobufResult;
use reflect::MessageDescriptor;
use repeated::RepeatedField;
use singular::{SingularField, SingularPtrField};
use stream::with_coded_output_stream_to_bytes;
use stream::CodedInputStream;
use stream::CodedOutputStream;
use stream::WithCodedInputStream;
use stream::WithCodedOutputStream;
use unknown::UnknownFields;

/// Trait implemented for all generated structs for protobuf messages.
///
/// Also, generated messages implement `Clone + Default + PartialEq`
pub trait Message: fmt::Debug + Clear + Any + Send + Sync {
    /// Message descriptor for this message, used for reflection.
    fn descriptor(&self) -> &'static MessageDescriptor;

    /// True iff all required fields are initialized.
    /// Always returns `true` for protobuf 3.
    fn is_initialized(&self) -> bool;

    /// Update this message object with fields read from given stream.
    fn merge_from(&mut self, is: &mut CodedInputStream) -> ProtobufResult<()>;

    /// Write message to the stream.
    ///
    /// Sizes of this messages and nested messages must be cached
    /// by calling `compute_size` prior to this call.
    fn write_to_with_cached_sizes(&self, os: &mut CodedOutputStream) -> ProtobufResult<()>;

    /// Compute and cache size of this message and all nested messages
    fn compute_size(&self) -> u32;

    /// Get size previously computed by `compute_size`.
    fn get_cached_size(&self) -> u32;

    /// Write the message to the stream.
    ///
    /// Results in error if message is not fully initialized.
    fn write_to(&self, os: &mut CodedOutputStream) -> ProtobufResult<()> {
        self.check_initialized()?;

        // cache sizes
        self.compute_size();
        // TODO: reserve additional
        self.write_to_with_cached_sizes(os)?;

        Ok(())
    }

    /// Write the message to the stream prepending the message with message length
    /// encoded as varint.
    fn write_length_delimited_to(&self, os: &mut CodedOutputStream) -> ProtobufResult<()> {
        let size = self.compute_size();
        os.write_raw_varint32(size)?;
        self.write_to_with_cached_sizes(os)?;

        // TODO: assert we've written same number of bytes as computed

        Ok(())
    }

    /// Write the message to the vec, prepend the message with message length
    /// encoded as varint.
    fn write_length_delimited_to_vec(&self, vec: &mut Vec<u8>) -> ProtobufResult<()> {
        let mut os = CodedOutputStream::vec(vec);
        self.write_length_delimited_to(&mut os)?;
        os.flush()?;
        Ok(())
    }

    /// Update this message object with fields read from given stream.
    fn merge_from_bytes(&mut self, bytes: &[u8]) -> ProtobufResult<()> {
        let mut is = CodedInputStream::from_bytes(bytes);
        self.merge_from(&mut is)
    }

    /// Check if all required fields of this object are initialized.
    fn check_initialized(&self) -> ProtobufResult<()> {
        if !self.is_initialized() {
            Err(ProtobufError::message_not_initialized(
                self.descriptor().name(),
            ))
        } else {
            Ok(())
        }
    }

    /// Write the message to the writer.
    fn write_to_writer(&self, w: &mut Write) -> ProtobufResult<()> {
        w.with_coded_output_stream(|os| self.write_to(os))
    }

    /// Write the message to bytes vec.
    fn write_to_vec(&self, v: &mut Vec<u8>) -> ProtobufResult<()> {
        v.with_coded_output_stream(|os| self.write_to(os))
    }

    /// Write the message to bytes vec.
    fn write_to_bytes(&self) -> ProtobufResult<Vec<u8>> {
        self.check_initialized()?;

        let size = self.compute_size() as usize;
        let mut v = Vec::with_capacity(size);
        // skip zerofill
        unsafe {
            v.set_len(size);
        }
        {
            let mut os = CodedOutputStream::bytes(&mut v);
            self.write_to_with_cached_sizes(&mut os)?;
            os.check_eof();
        }
        Ok(v)
    }

    /// Write the message to the writer, prepend the message with message length
    /// encoded as varint.
    fn write_length_delimited_to_writer(&self, w: &mut Write) -> ProtobufResult<()> {
        w.with_coded_output_stream(|os| self.write_length_delimited_to(os))
    }

    /// Write the message to the bytes vec, prepend the message with message length
    /// encoded as varint.
    fn write_length_delimited_to_bytes(&self) -> ProtobufResult<Vec<u8>> {
        with_coded_output_stream_to_bytes(|os| self.write_length_delimited_to(os))
    }

    /// Get a reference to unknown fields.
    fn get_unknown_fields<'s>(&'s self) -> &'s UnknownFields;
    /// Get a mutable reference to unknown fields.
    fn mut_unknown_fields<'s>(&'s mut self) -> &'s mut UnknownFields;

    /// Get type id for downcasting.
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    /// View self as `Any`.
    fn as_any(&self) -> &Any;

    /// View self as mutable `Any`.
    fn as_any_mut(&mut self) -> &mut Any {
        panic!()
    }

    /// Convert boxed self to boxed `Any`.
    fn into_any(self: Box<Self>) -> Box<Any> {
        panic!()
    }

    // Rust does not allow implementation of trait for trait:
    // impl<M : Message> fmt::Debug for M {
    // ...
    // }

    /// Create an empty message object.
    ///
    ///
    /// ```
    /// # use protobuf::Message;
    /// # fn foo<MyMessage: Message>() {
    /// let m = MyMessage::new();
    /// # }
    /// ```
    fn new() -> Self
    where
        Self: Sized;

    /// Get message descriptor for message type.
    ///
    /// ```
    /// # use protobuf::Message;
    /// # fn foo<MyMessage: Message>() {
    /// let descriptor = MyMessage::descriptor_static();
    /// assert_eq!("MyMessage", descriptor.name());
    /// # }
    /// ```
    fn descriptor_static() -> &'static MessageDescriptor
    where
        Self: Sized,
    {
        panic!(
            "descriptor_static is not implemented for message, \
             LITE_RUNTIME must be used"
        );
    }

    /// Return a pointer to default immutable message with static lifetime.
    ///
    /// ```
    /// # use protobuf::Message;
    /// # fn foo<MyMessage: Message>() {
    /// let m: &MyMessage = MyMessage::default_instance();
    /// # }
    /// ```
    fn default_instance() -> &'static Self
    where
        Self: Sized;
}

pub fn message_down_cast<'a, M: Message + 'a>(m: &'a Message) -> &'a M {
    m.as_any().downcast_ref::<M>().unwrap()
}

/// Parse message from stream.
pub fn parse_from<M: Message>(is: &mut CodedInputStream) -> ProtobufResult<M> {
    let mut r: M = Message::new();
    r.merge_from(is)?;
    r.check_initialized()?;
    Ok(r)
}

/// Parse message from reader.
/// Parse stops on EOF or when error encountered.
pub fn parse_from_reader<M: Message>(reader: &mut Read) -> ProtobufResult<M> {
    reader.with_coded_input_stream(|is| parse_from::<M>(is))
}

/// Parse message from byte array.
pub fn parse_from_bytes<M: Message>(bytes: &[u8]) -> ProtobufResult<M> {
    bytes.with_coded_input_stream(|is| parse_from::<M>(is))
}

/// Parse message from `Bytes` object.
/// Resulting message may share references to the passed bytes object.
#[cfg(feature = "bytes")]
pub fn parse_from_carllerche_bytes<M: Message>(bytes: &Bytes) -> ProtobufResult<M> {
    // Call trait explicitly to avoid accidental construction from `&[u8]`
    WithCodedInputStream::with_coded_input_stream(bytes, |is| parse_from::<M>(is))
}

/// Parse length-delimited message from stream.
///
/// Read varint length first, and read messages of that length then.
///
/// This function is deprecated and will be removed in the next major release.
#[deprecated]
pub fn parse_length_delimited_from<M: Message>(is: &mut CodedInputStream) -> ProtobufResult<M> {
    is.read_message::<M>()
}

/// Parse length-delimited message from `Read`.
///
/// This function is deprecated and will be removed in the next major release.
#[deprecated]
pub fn parse_length_delimited_from_reader<M: Message>(r: &mut Read) -> ProtobufResult<M> {
    // TODO: wrong: we may read length first, and then read exact number of bytes needed
    r.with_coded_input_stream(|is| is.read_message::<M>())
}

/// Parse length-delimited message from bytes.
///
/// This function is deprecated and will be removed in the next major release.
#[deprecated]
pub fn parse_length_delimited_from_bytes<M: Message>(bytes: &[u8]) -> ProtobufResult<M> {
    bytes.with_coded_input_stream(|is| is.read_message::<M>())
}

/// A trait used for pretty printing protobuf message.
pub trait PbPrint {
    /// Format `self` to `buf`.
    fn fmt(&self, name: &str, buf: &mut String);
}

pub fn escape(data: &[u8], buf: &mut String) {
    buf.reserve(data.len() + 2);
    buf.push('"');
    for &c in data {
        match c {
            b'\n' => buf.push_str(r"\n"),
            b'\r' => buf.push_str(r"\r"),
            b'\t' => buf.push_str(r"\t"),
            b'"' => buf.push_str("\\\""),
            b'\\' => buf.push_str(r"\\"),
            _ => {
                if c >= 0x20 && c < 0x7f {
                    // c is printable
                    buf.push(c as char);
                } else {
                    buf.push('\\');
                    buf.push((b'0' + (c >> 6)) as char);
                    buf.push((b'0' + ((c >> 3) & 7)) as char);
                    buf.push((b'0' + (c & 7)) as char);
                }
            }
        }
    }
    buf.push('"');
}

pub fn hex_escape(data: &[u8], buf: &mut String) {
    hex::ToHex::write_hex_upper(&data, buf).unwrap();
}

#[inline]
pub fn push_start(name: &str, buf: &mut String) {
    if !buf.is_empty() {
        buf.push(' ');
    }
    buf.push_str(name);
}

/// Push name to buf.
#[inline]
pub fn push_message_start(name: &str, buf: &mut String) {
    push_start(name, buf);
    buf.push_str(" {");
}

/// Push name to buf.
#[inline]
pub fn push_field_start(name: &str, buf: &mut String) {
    push_start(name, buf);
    if !name.is_empty() {
        buf.push_str(": ");
    }
}

impl<T: PbPrint> PbPrint for Option<T> {
    #[inline]
    fn fmt(&self, name: &str, buf: &mut String) {
        match self.as_ref() {
            None => return,
            Some(v) => v.fmt(name, buf),
        }
    }
}

impl PbPrint for String {
    #[inline]
    fn fmt(&self, name: &str, buf: &mut String) {
        if self.is_empty() {
            return;
        }
        push_field_start(name, buf);
        escape(self.as_bytes(), buf);
    }
}

impl PbPrint for Vec<u8> {
    #[inline]
    fn fmt(&self, name: &str, buf: &mut String) {
        if self.is_empty() {
            return;
        }
        push_field_start(name, buf);
        hex_escape(self, buf);
    }
}

impl<T: PbPrint> PbPrint for Vec<T> {
    #[inline]
    fn fmt(&self, name: &str, buf: &mut String) {
        if self.is_empty() {
            return;
        }
        for v in self {
            v.fmt(name, buf);
        }
    }
}

impl<K: PbPrint, V: PbPrint, S> PbPrint for HashMap<K, V, S> {
    #[inline]
    fn fmt(&self, name: &str, buf: &mut String) {
        if self.is_empty() {
            return;
        }
        push_field_start(name, buf);
        buf.push_str("map[");
        for (k, v) in self {
            k.fmt("", buf);
            buf.push(':');
            v.fmt("", buf);
        }
        buf.push_str(" ]");
    }
}

macro_rules! print_number {
    ($t:ty, $zero:expr) => {
        impl PbPrint for $t {
            #[inline]
            fn fmt(&self, name: &str, buf: &mut String) {
                if *self == $zero {
                    return;
                }
                push_field_start(name, buf);
                write!(buf, "{}", self).unwrap();
            }
        }
    };
}

print_number!(i32, 0);
print_number!(i64, 0);
print_number!(u32, 0);
print_number!(u64, 0);
print_number!(f32, 0.0);
print_number!(f64, 0.0);

impl PbPrint for bool {
    #[inline]
    fn fmt(&self, name: &str, buf: &mut String) {
        if !*self {
            return;
        }
        push_field_start(name, buf);
        buf.push_str("true");
    }
}

impl<T: PbPrint> PbPrint for RepeatedField<T> {
    fn fmt(&self, name: &str, buf: &mut String) {
        if self.is_empty() {
            return;
        }
        for v in self {
            v.fmt(name, buf);
        }
    }
}

impl<T: PbPrint> PbPrint for SingularPtrField<T> {
    #[inline]
    fn fmt(&self, name: &str, buf: &mut String) {
        match self.as_ref() {
            None => return,
            Some(v) => v.fmt(name, buf),
        }
    }
}

impl<T: PbPrint> PbPrint for SingularField<T> {
    #[inline]
    fn fmt(&self, name: &str, buf: &mut String) {
        match self.as_ref() {
            None => return,
            Some(v) => v.fmt(name, buf),
        }
    }
}

/// Impl `PbPrint` if there is a `Debug` impl.
#[macro_export]
macro_rules! debug_to_pb_print {
    ($t:ty) => {
        impl PbPrint for $t {
            #[inline]
            fn fmt(&self, name: &str, buf: &mut String) {
                ::protobuf::push_message_start(name, buf);
                let old_len = buf.len();
                write!(buf, "{:?}", self).unwrap();
                if buf.len() > old_len {
                    buf.push_str(" }");
                } else {
                    buf.push('}');
                }
            }
        }
    };
}
