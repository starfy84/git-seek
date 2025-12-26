#[derive(Debug, Clone)]
pub struct Repository {
    name: String,
}

impl Repository {
    pub fn new(name: String) -> Self {
        Repository { name}
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}