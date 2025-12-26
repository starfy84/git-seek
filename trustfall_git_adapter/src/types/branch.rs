#[derive(Debug, Clone)]
pub struct Branch {
    name: String,
}

impl Branch {
    pub fn new(name: String) -> Self {
        Branch { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}