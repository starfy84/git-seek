use git2::Commit as Git2Commit;

#[derive(Debug, Clone)]
pub struct Commit<'a> {
    commit: Git2Commit<'a>,
}

impl<'a> Commit<'a> {
    pub fn new(commit: Git2Commit<'a>) -> Self {
        Self { commit }
    }

    pub fn inner(&self) -> &Git2Commit<'a> {
        &self.commit
    }

}