use git2::Oid;

#[derive(Debug, Clone)]
pub struct Tag {
    name: String,
    target_oid: Oid,
    message: Option<String>,
    tagger_name: Option<String>,
    tagger_email: Option<String>,
}

impl Tag {
    pub fn new(
        name: String,
        target_oid: Oid,
        message: Option<String>,
        tagger_name: Option<String>,
        tagger_email: Option<String>,
    ) -> Self {
        Self {
            name,
            target_oid,
            message,
            tagger_name,
            tagger_email,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn target_oid(&self) -> Oid {
        self.target_oid
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn tagger_name(&self) -> Option<&str> {
        self.tagger_name.as_deref()
    }

    pub fn tagger_email(&self) -> Option<&str> {
        self.tagger_email.as_deref()
    }
}
