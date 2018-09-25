use bytes::{BufMut, BytesMut};
use error::NatsError;
use protocol::{CommandError, Op};
use tokio_codec::{Decoder, Encoder};

#[derive(Default, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct OpCodec {
    next_index: usize,
}

impl OpCodec {
    pub fn new() -> Self {
        OpCodec::default()
    }
}

impl Encoder for OpCodec {
    type Error = NatsError;
    type Item = Op;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let buf = item.into_bytes()?;
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
    type Error = NatsError;
    type Item = Op;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Let's check if we find a blank space at the beginning
        if let Some(command_offset) = buf[self.next_index..]
            .iter()
            .position(|b| *b == b' ' || *b == b'\t' || *b == b'\n')
        {
            match Op::from_bytes(&buf[..command_offset], &buf) {
                Err(CommandError::IncompleteCommandError) => {
                    self.next_index = buf.len();
                    Ok(None)
                }
                Ok(maybe_op) => match maybe_op {
                    None => {
                        self.next_index = buf.len();
                        Ok(None)
                    }
                    Some(op) => {
                        buf.clear(); // haha, that's what was missing!
                        self.next_index = 0;
                        Ok(Some(op))
                    }
                },
                Err(e) => {
                    self.next_index = 0;
                    Err(e.into())
                }
            }
        } else {
            // First blank not found yet, continuing
            self.next_index = buf.len();
            Ok(None)
        }
    }
}
