use protocol::{Command, CommandError};

#[derive(Debug, Clone, Builder)]
pub struct UnsubCommand {
    pub sid: String,
    pub max_msgs: Option<u32>,
}

impl Command for UnsubCommand {
    fn into_vec(self) -> Result<Vec<u8>, CommandError> {
        let mm = if let Some(max_msgs) = self.max_msgs {
            format!("\t{}", max_msgs)
        } else {
            "".into()
        };

        Ok(format!("UNSUB\t{}{}\r\n", self.sid, mm).as_bytes().to_vec())
    }
}
