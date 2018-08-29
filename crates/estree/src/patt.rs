use easter::patt::Patt;
use easter::id::{Id, IdExt};
use unjson::ty::{Object};

use id::IntoId;
use result::Result;
use tag::{Tag, TagOf};

use node::ExtractNode;

pub trait IntoPatt {
    fn into_patt(self) -> Result<Patt<Id>>;
}

impl IntoPatt for Object {
    fn into_patt(mut self) -> Result<Patt<Id>> {
        let tag = self.tag()?;
        match tag {
            Tag::AssignmentPattern => {
                let left = self.extract_id("left")?;
                let right = Box::new(self.extract_expr("right")?);
                Ok(Patt::Assign(None, left, right))
            }
            _ => self.into_id().map(|id| id.into_patt())
        }
    }
}
