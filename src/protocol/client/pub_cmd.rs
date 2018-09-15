use bytes::{BufMut, Bytes, BytesMut};
use protocol::{Command, CommandError};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct PubCommand {
    pub subject: String,
    pub reply_to: Option<String>,
    pub payload: Bytes,
}

impl Command for PubCommand {
    fn into_vec(self) -> Result<Bytes, CommandError> {
        let rt = if let Some(reply_to) = self.reply_to {
            format!("\t{}", reply_to)
        } else {
            "".into()
        };

        let cmd_str = format!("PUB\t{}{}\t{}\r\n", self.subject, rt, self.payload.len());
        let mut bytes = BytesMut::new();
        bytes.put(cmd_str.as_bytes());
        bytes.put(self.payload);
        bytes.put("\r\n");

        Ok(bytes.freeze())
    }
}

impl PubCommandBuilder {
    pub fn auto_reply_to(mut self) -> Self {
        let mut rng = thread_rng();
        let inbox = rng.sample_iter(&Alphanumeric).take(16).collect();
        self.reply_to = Some(Some(inbox));
        self
    }

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
