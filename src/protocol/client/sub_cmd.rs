use bytes::Bytes;
use protocol::{Command, CommandError};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(skip))]
pub struct SubCommand {
    pub subject: String,
    #[builder(default)]
    pub queue_group: Option<String>,
    #[builder(default = "Ok(Self::generate_sid())")]
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

        Ok(format!("SUB\t{}{}\t{}", self.subject, qg, self.sid).as_bytes().into())
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
        if let Some(subj) = self.subject.as_ref() {
            check_cmd_arg!(subj, "subject");
        }

        if let Some(qg_maybe) = self.queue_group.as_ref() {
            if let Some(qg) = qg_maybe {
                check_cmd_arg!(qg, "queue group");
            }
        }

        Ok(())
    }

    pub fn build(self) -> Result<SubCommand, String> {
        self.validate()?;

        Ok(SubCommand {
            subject: Clone::clone(self.subject.as_ref().ok_or("subject must be initialized")?),
            queue_group: Clone::clone(self.queue_group.as_ref().ok_or("queue_group must be initialized")?),
            sid: Clone::clone(self.sid.as_ref().ok_or("sid must be initialized")?),
        })
    }
}
