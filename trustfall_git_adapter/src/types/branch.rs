use git2::Branch as Git2Branch;
use std::rc::Rc;

#[derive(Clone)]
pub struct Branch<'a> {
    branch: Rc<Git2Branch<'a>>,
}

impl<'a> Branch<'a> {
    pub fn new(branch: Git2Branch<'a>) -> Self {
        Branch { 
            branch: Rc::new(branch)
        }
    }

    pub fn inner(&self) -> &Git2Branch<'a> {
        &self.branch
    }
}

impl<'a> std::fmt::Debug for Branch<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.branch.name()
            .unwrap_or(None)
            .unwrap_or("<unnamed>");

        f.debug_struct("Branch")
            .field("name", &name)
            .finish()
    }
}