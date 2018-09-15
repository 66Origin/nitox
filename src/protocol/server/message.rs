use bytes::{BufMut, Bytes, BytesMut};
use protocol::{Command, CommandError};

#[derive(Debug, Clone, PartialEq, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct Message {
    pub subject: String,
    pub sid: String,
    pub reply_to: Option<String>,
    pub payload: Bytes,
}

impl Command for Message {
    fn into_vec(self) -> Result<Vec<u8>, CommandError> {
        let rt = if let Some(reply_to) = self.reply_to {
            format!("\t{}", reply_to)
        } else {
            "".into()
        };

        let cmd_str = format!(
            "MSG\t{}\t{}{}\t{}\r\n",
            self.subject,
            self.sid,
            rt,
            self.payload.len()
        );
        let mut bytes = BytesMut::new();
        bytes.put(cmd_str.as_bytes());
        bytes.put(self.payload);
        bytes.put("\r\n");

        Ok(bytes.to_vec())
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
