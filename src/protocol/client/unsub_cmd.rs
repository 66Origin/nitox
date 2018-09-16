use bytes::Bytes;
use protocol::{Command, CommandError};

#[derive(Debug, Clone, Builder)]
pub struct UnsubCommand {
    pub sid: String,
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
