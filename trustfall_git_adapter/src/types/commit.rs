#[derive(Debug, Clone)]
pub struct Commit {
    hash: String,
    message: Option<String>,
}

impl Commit {
    pub fn new(hash: String, message: Option<String>) -> Self {
        Commit { hash, message }
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

}