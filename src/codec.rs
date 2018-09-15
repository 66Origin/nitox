use bytes::{BufMut, BytesMut, IntoBuf};
use error::NatsError;
use protocol::Op;
use tokio_codec::{Decoder, Encoder};

pub struct OpCodec;

impl Encoder for OpCodec {
    type Item = Op;
    type Error = NatsError;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let buf = item.to_bytes()?;
        let buf_len = buf.len();
        let remaining_bytes = dst.remaining_mut();
        if remaining_bytes < buf_len {
            dst.reserve(buf_len);
        }
        dst.put(buf);
        Ok(())
    }
}

impl Decoder for OpCodec {
    type Item = Op;
    type Error = NatsError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        Ok(Some(Self::Item::from_bytes(src)?))
    }
}
