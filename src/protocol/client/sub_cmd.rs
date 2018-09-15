use protocol::{Command, CommandError};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

#[derive(Debug, Clone, Builder)]
#[builder(build_fn(skip))]
pub struct SubCommand {
    pub subject: String,
    pub queue_group: Option<String>,
    pub sid: String,
}

impl Command for SubCommand {
    fn into_vec(self) -> Result<Vec<u8>, CommandError> {
        let qg = if let Some(queue_group) = self.queue_group {
            format!("\t{}", queue_group)
        } else {
            "".into()
        };

        Ok(format!("SUB\t{}{}\t{}", self.subject, qg, self.sid)
            .as_bytes()
            .to_vec())
    }
}

impl SubCommandBuilder {
    fn validate(&self) -> Result<(), String> {
        if let Some(ref subj) = self.subject {
            check_cmd_arg!(subj, "subject");
        }

        if let Some(ref qg_maybe) = self.queue_group {
            if let Some(qg) = qg_maybe {
                check_cmd_arg!(qg, "queue group");
            }
        }

        Ok(())
    }

    pub fn build(mut self) -> Result<SubCommand, String> {
        let _ = self.validate()?;

        if self.sid.is_none() {
            let mut rng = thread_rng();
            let sid: String = rng.sample_iter(&Alphanumeric).take(8).collect();

            self.sid = Some(sid);
        }

        Ok(SubCommand {
            subject: Clone::clone(self.subject.as_ref().ok_or("subject must be initialized")?),
            queue_group: Clone::clone(
                self.queue_group
                    .as_ref()
                    .ok_or("queue_group must be initialized")?,
            ),
            sid: Clone::clone(self.sid.as_ref().ok_or("sid must be initialized")?),
        })
    }
}
