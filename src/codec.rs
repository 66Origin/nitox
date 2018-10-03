use bytes::{BufMut, BytesMut};
use error::NatsError;
use protocol::{CommandError, Op};
use tokio_codec::{Decoder, Encoder};

/// `tokio-codec` implementation of the protocol parsing
#[derive(Default, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct OpCodec {
    /// Used as an optimization for buffer lookup
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
        if buf.is_empty() {
            return Ok(None);
        }

        debug!(target: "nitox", "codec buffer is {:?}", buf);
        // Let's check if we find a blank space at the beginning
        if let Some(command_offset) = buf[self.next_index..]
            .iter()
            .position(|b| *b == b' ' || *b == b'\t' || *b == b'\r')
        {
            let command_end = self.next_index + command_offset;
            debug!(target: "nitox", "codec detected command name {:?}", &buf[..command_end]);

            if let Some(command_body_offset) = buf[command_end..].windows(2).position(|w| w == b"\r\n") {
                let mut end_buf_pos = command_end + command_body_offset + 2;

                if &buf[..command_end] == b"PUB" || &buf[..command_end] == b"MSG" {
                    debug!(target: "nitox", "detected PUB or MSG, looking for second CRLF");
                    if let Some(new_end) = buf[end_buf_pos..].windows(2).position(|w| w == b"\r\n") {
                        debug!(target: "nitox", "found second CRLF at position {}", end_buf_pos + new_end + 2);
                        end_buf_pos += new_end + 2;
                    } else {
                        debug!(target: "nitox", "command was incomplete");
                        return Ok(None);
                    }
                }

                debug!(target: "nitox", "codec detected command body {:?}", &buf[..end_buf_pos]);
                match Op::from_bytes(&buf[..command_end], &buf[..end_buf_pos]) {
                    Err(CommandError::IncompleteCommandError) => {
                        debug!(target: "nitox", "command was incomplete");
                        self.next_index = buf.len();
                        Ok(None)
                    }
                    Ok(op) => {
                        debug!(target: "nitox", "codec parsed command {:#?}", op);
                        let _ = buf.split_to(end_buf_pos);
                        debug!(target: "nitox", "buffer now contains {:?}", buf);
                        self.next_index = 0;
                        Ok(Some(op))
                    }
                    Err(e) => {
                        debug!(target: "nitox", "command couldn't be parsed {}", e);
                        self.next_index = 0;
                        Err(e.into())
                    }
                }
            } else {
                Ok(None)
            }
        } else {
            // First blank not found yet, continuing
            debug!(target: "nitox", "no whitespace found yet, continuing");
            self.next_index = buf.len();
            Ok(None)
        }
    }
}
