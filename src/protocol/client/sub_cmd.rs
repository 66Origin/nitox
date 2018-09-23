use bytes::Bytes;
use protocol::{Command, CommandError};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct SubCommand {
    #[builder(setter(into))]
    pub subject: String,
    #[builder(default)]
    pub queue_group: Option<String>,
    #[builder(setter(into), default = "Self::generate_sid()")]
    pub sid: String,
}

impl Command for SubCommand {
    const CMD_NAME: &'static [u8] = b"SUB";

    fn into_vec(self) -> Result<Bytes, CommandError> {
        let qg = if let Some(queue_group) = self.queue_group {
            format!("\t{}", queue_group)
        } else {
            "".into()
        };

        Ok(format!("SUB\t{}{}\t{}\r\n", self.subject, qg, self.sid)
            .as_bytes()
            .into())
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

        // Extract subject
        let subject: String = split.next().ok_or_else(|| CommandError::CommandMalformed)?.into();

        let sid: String = split.next_back().ok_or_else(|| CommandError::CommandMalformed)?.into();

        let queue_group: Option<String> = split.next().map(|v| v.into());

        Ok(SubCommand {
            subject,
            queue_group,
            sid,
        })
    }
}

impl SubCommandBuilder {
    pub fn generate_sid() -> String {
        thread_rng().sample_iter(&Alphanumeric).take(8).collect()
    }

    fn validate(&self) -> Result<(), String> {
        if let Some(ref subj) = self.subject {
            check_cmd_arg!(subj, "subject");
        }

        if let Some(ref qg_maybe) = self.queue_group {
            if let Some(ref qg) = qg_maybe {
                check_cmd_arg!(qg, "queue group");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod sub_command_tests {
    use super::{SubCommand, SubCommandBuilder};
    use protocol::Command;

    static DEFAULT_SUB: &'static str = "SUB\tFOO\tpouet\r\n";

    #[test]
    fn it_parses() {
        let parse_res = SubCommand::try_parse(DEFAULT_SUB.as_bytes());
        assert!(parse_res.is_ok());
        let cmd = parse_res.unwrap();
        assert_eq!(&cmd.subject, "FOO");
        assert_eq!(&cmd.sid, "pouet")
    }

    #[test]
    fn it_stringifies() {
        let cmd = SubCommandBuilder::default()
            .subject("FOO")
            .sid("pouet")
            .build()
            .unwrap();

        let cmd_bytes_res = cmd.into_vec();
        assert!(cmd_bytes_res.is_ok());
        let cmd_bytes = cmd_bytes_res.unwrap();

        assert_eq!(DEFAULT_SUB, cmd_bytes);
    }
}
