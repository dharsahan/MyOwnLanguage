use std::fmt;

#[derive(Debug, Clone)]
pub enum LangError {
    LexError { message: String },
    ParseError { message: String },
    RuntimeError { message: String },
}

impl fmt::Display for LangError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LangError::LexError { message } => write!(f, "Lex Error: {}", message),
            LangError::ParseError { message } => write!(f, "Parse Error: {}", message),
            LangError::RuntimeError { message } => write!(f, "Runtime Error: {}", message),
        }
    }
}

impl std::error::Error for LangError {}

impl LangError {
    pub fn lex(msg: impl Into<String>) -> Self {
        LangError::LexError { message: msg.into() }
    }

    pub fn parse(msg: impl Into<String>) -> Self {
        LangError::ParseError { message: msg.into() }
    }

    pub fn runtime(msg: impl Into<String>) -> Self {
        LangError::RuntimeError { message: msg.into() }
    }
}
