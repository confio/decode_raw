use protofish::prelude::{Context, FieldValue};

pub fn decode_fields(bytes: &[u8]) -> Vec<FieldValue> {
    let context = Context::parse(&[r#"
        syntax = "proto3";
        package Proto;

        message Empty { }
    "#])
    .unwrap();

    let request = context.get_message("Proto.Empty").unwrap();
    let value = request.decode(bytes, &context);
    value.fields
}
