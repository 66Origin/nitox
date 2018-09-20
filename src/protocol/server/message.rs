use bytes::{BufMut, Bytes, BytesMut};
use protocol::{Command, CommandError};

#[derive(Debug, Clone, PartialEq, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct Message {
    #[builder(setter(into))]
    pub subject: String,
    #[builder(setter(into))]
    pub sid: String,
    #[builder(default)]
    pub reply_to: Option<String>,
    #[builder(setter(into))]
    pub payload: Bytes,
}

impl Command for Message {
    const CMD_NAME: &'static [u8] = b"MSG";

    fn into_vec(self) -> Result<Bytes, CommandError> {
        let rt = if let Some(reply_to) = self.reply_to {
            format!("\t{}", reply_to)
        } else {
            "".into()
        };

        let cmd_str = format!("MSG\t{}\t{}{}\t{}\r\n", self.subject, self.sid, rt, self.payload.len());
        let mut bytes = BytesMut::new();
        bytes.put(cmd_str.as_bytes());
        bytes.put(self.payload);
        bytes.put("\r\n");

        Ok(bytes.freeze())
    }

    fn try_parse(buf: &[u8]) -> Result<Self, CommandError> {
        let len = buf.len();

        if buf[len - 2..] != [b'\r', b'\n'] {
            return Err(CommandError::IncompleteCommandError);
        }

        if let Some(payload_start) = buf[..len - 2].iter().position(|b| *b == b'\r') {
            if buf[payload_start + 1] != b'\n' {
                return Err(CommandError::CommandMalformed);
            }

            let payload: Bytes = buf[payload_start + 2..len - 2].into();

            let whole_command = ::std::str::from_utf8(&buf[..payload_start])?;
            let mut split = whole_command.split_whitespace();
            let cmd = split.next().ok_or_else(|| CommandError::CommandMalformed)?;
            // Check if we're still on the right command
            if cmd.as_bytes() != Self::CMD_NAME {
                return Err(CommandError::CommandMalformed);
            }

            let payload_len: usize = split
                .next_back()
                .ok_or_else(|| CommandError::CommandMalformed)?
                .parse()?;

            if payload.len() != payload_len {
                return Err(CommandError::CommandMalformed);
            }

            // Extract subject
            let subject: String = split.next().ok_or_else(|| CommandError::CommandMalformed)?.into();

            let sid: String = split.next().ok_or_else(|| CommandError::CommandMalformed)?.into();

            let reply_to: Option<String> = split.next().map(|v| v.into());

            Ok(Message {
                subject,
                sid,
                payload,
                reply_to,
            })
        } else {
            Err(CommandError::CommandMalformed)
        }
    }
}

impl MessageBuilder {
    fn validate(&self) -> Result<(), String> {
        if let Some(ref subj) = self.subject {
            check_cmd_arg!(subj, "subject");
        }

        if let Some(ref reply_to_maybe) = self.reply_to {
            if let Some(ref reply_to) = reply_to_maybe {
                check_cmd_arg!(reply_to, "inbox");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod message_command_tests {
    use super::{Message, MessageBuilder};
    use protocol::Command;

    static DEFAULT_MSG: &'static str = "MSG\tFOO\tpouet\t4\r\ntoto\r\n";

    #[test]
    fn it_parses() {
        let parse_res = Message::try_parse(DEFAULT_MSG.as_bytes());
        assert!(parse_res.is_ok());
        let cmd = parse_res.unwrap();
        assert!(cmd.reply_to.is_none());
        assert_eq!(&cmd.subject, "FOO");
        assert_eq!(&cmd.sid, "pouet");
        assert_eq!(cmd.payload, "toto");
    }

    #[test]
    fn it_stringifies() {
        let cmd = MessageBuilder::default()
            .subject("FOO")
            .sid("pouet")
            .payload("toto")
            .build()
            .unwrap();

        let cmd_bytes_res = cmd.into_vec();
        assert!(cmd_bytes_res.is_ok());
        let cmd_bytes = cmd_bytes_res.unwrap();

        assert_eq!(DEFAULT_MSG, cmd_bytes);
    }
}
