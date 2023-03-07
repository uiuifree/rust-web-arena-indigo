use std::fmt::{Debug, Formatter};

pub enum WebArenaIndigoApiError {
    Connection(String),
    Status(u16, String),
    JsonParse(String),
}

impl WebArenaIndigoApiError {
    pub fn to_string(&self) -> String {
        match self {
            WebArenaIndigoApiError::Connection(e) => { e.to_string() }
            WebArenaIndigoApiError::JsonParse(e) => { format!("json parse : {}",e.to_string()) }
            WebArenaIndigoApiError::Status(status, value) => {
                format!("status: {} ,message:{}", status, value)
            }
        }
    }
}

impl Debug for WebArenaIndigoApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_string().as_str())
    }
}
