pub struct BotReply {
    messages: Vec<String>,
}

impl BotReply {
    pub fn new() -> Self {
        BotReply {
            messages: Vec::new(),
        }
    }

    pub fn add(&mut self, msg: String) {
        self.messages.push(msg);
    }

    pub fn add_boxed(&mut self, msg: String) {
        self.messages.push(format!("```\n{}\n```", msg));
    }

    pub fn to_string(&self) -> String {
        self.messages.join("\n")
    }
}
