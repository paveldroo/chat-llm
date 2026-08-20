use crate::{error::Error, llm::Message};

#[derive(Default)]
pub struct Conversation {
    messages: Vec<Message>,
}

impl Conversation {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_system(&mut self, content: &str) -> Result<(), Error> {
        if !self.messages.is_empty() {
            return Err(Error::SystemMessage);
        }
        self.messages.push(Message::system(content));
        Ok(())
    }

    pub fn push_user(&mut self, content: &str) {
        self.messages.push(Message::user(content));
    }

    pub fn push_assistant(&mut self, content: &str) {
        self.messages.push(Message::assistant(content));
    }

    #[must_use]
    pub const fn as_slice(&self) -> &[Message] {
        self.messages.as_slice()
    }
}
