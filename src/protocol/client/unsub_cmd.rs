use bytes::Bytes;
use protocol::{Command, CommandError};

#[derive(Debug, Clone, PartialEq, Builder)]
pub struct UnsubCommand {
    #[builder(setter(into))]
    pub sid: String,
    #[builder(default)]
    pub max_msgs: Option<u32>,
}

impl Command for UnsubCommand {
    const CMD_NAME: &'static [u8] = b"UNSUB";

    fn into_vec(self) -> Result<Bytes, CommandError> {
        let mm = if let Some(max_msgs) = self.max_msgs {
            format!("\t{}", max_msgs)
        } else {
            "".into()
        };

        Ok(format!("UNSUB\t{}{}\r\n", self.sid, mm).as_bytes().into())
    }

    fn try_parse(buf: &[u8]) -> Result<Self, CommandError> {
        let len = buf.len();

        if buf[len - 2..] != [b'\r', b'\n'] {
            return Err(CommandError::IncompleteCommandError);
        }

        let whole_command = ::std::str::from_utf8(&buf[..len - 2])?;
        let mut split = whole_command.split_whitespace();
        let cmd = split.next().ok_or_else(|| CommandError::CommandMalformed)?;
        // Check if we're still on the right command
        if cmd.as_bytes() != Self::CMD_NAME {
            return Err(CommandError::CommandMalformed);
        }

        let sid: String = split.next().ok_or_else(|| CommandError::CommandMalformed)?.into();

        let max_msgs: Option<u32> = if let Some(mm) = split.next() {
            Some(mm.parse()?)
        } else {
            None
        };

        Ok(UnsubCommand { sid, max_msgs })
    }
}

#[cfg(test)]
mod unsub_command_tests {
    use super::{UnsubCommand, UnsubCommandBuilder};
    use protocol::Command;

    static DEFAULT_UNSUB: &'static str = "UNSUB\tpouet\r\n";

    #[test]
    fn it_parses() {
        let parse_res = UnsubCommand::try_parse(DEFAULT_UNSUB.as_bytes());
        assert!(parse_res.is_ok());
        let cmd = parse_res.unwrap();
        assert_eq!(&cmd.sid, "pouet");
        assert!(cmd.max_msgs.is_none());
    }

    #[test]
    fn it_stringifies() {
        let cmd = UnsubCommandBuilder::default().sid("pouet").build().unwrap();

        let cmd_bytes_res = cmd.into_vec();
        assert!(cmd_bytes_res.is_ok());
        let cmd_bytes = cmd_bytes_res.unwrap();

        assert_eq!(DEFAULT_UNSUB, cmd_bytes);
    }
}
