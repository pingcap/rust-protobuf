// Ensure kvproto's generated messages use the protobuf runtime from this checkout.
pub fn serialize_context(context: &kvproto::kvrpcpb::Context) -> protobuf::ProtobufResult<Vec<u8>> {
    protobuf::Message::write_to_bytes(context)
}
