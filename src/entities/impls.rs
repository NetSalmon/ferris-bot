use crate::entities::{MarkedMessage, Message, Request, Tool};

pub struct RequestBuilder {
    pub model: String,
    pub messages: Vec<Message>,
    pub tools: Option<Vec<Tool>>,
    pub tool_choice: Option<String>,
    pub temperature: Option<f32>,
    pub stream: Option<bool>,
    pub max_tokens: Option<u32>,
    pub enable_thinking: Option<bool>,
}

impl Request {
    pub fn builder() -> RequestBuilder {
        RequestBuilder {
            model: String::new(),
            messages: vec![],
            tools: None,
            tool_choice: None,
            temperature: None,
            stream: None,
            max_tokens: None,
            enable_thinking: None,
        }
    }

    pub fn push_message(&mut self, message: Message) {
        let marked = MarkedMessage {id: self.messages.len(), message};
        self.messages.push(marked);
    }
    
    pub fn push_tool(mut self, tool: Tool) {
        match &mut self.tools {
            Some(tools) => tools.push(tool),
            None => self.tools = Some(vec![tool]),
        }
    }
}

impl RequestBuilder {
    pub fn build(self) -> Request {
        Request {
            model: self.model,
            messages: self.messages.into_iter().enumerate().map(|(id, message)| MarkedMessage {id, message}).collect(),
            tools: self.tools,
            tool_choice: self.tool_choice,
            temperature: self.temperature,
            stream: self.stream,
            max_tokens: self.max_tokens,
            enable_thinking: self.enable_thinking,
        }
    }

    pub fn set_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    pub fn push_message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    pub fn push_tool(mut self, tool: Tool) -> Self {
        match &mut self.tools {
            Some(tools) => tools.push(tool),
            None => self.tools = Some(vec![tool]),
        }
        self
    }

    pub fn set_tool_choice(mut self, choice: String) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    pub fn set_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn enable_thinking(mut self) -> Self {
        self.enable_thinking = Some(true);
        self
    }
    
    pub fn disable_thinking(mut self) -> Self {
        self.enable_thinking = Some(false);
        self
    }

    pub fn enable_stream(mut self) -> Self {
        self.stream = Some(true);
        self
    }
    
    pub fn disable_stream(mut self) -> Self {
        self.stream = Some(false);
        self
    }

    pub fn set_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
}