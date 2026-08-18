use crate::llm::Message;

#[derive(Default)]
pub struct Conversation {
    messages: Vec<Message>,
}

impl Conversation {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    #[must_use]
    pub const fn as_slice(&self) -> &[Message] {
        self.messages.as_slice()
    }
}
