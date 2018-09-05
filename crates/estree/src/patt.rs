use easter::patt::{Patt, RestPatt, CompoundPatt};
use easter::id::{Id, IdExt};
use unjson::ty::Object;
use unjson::{ExtractField, Unjson};

use id::IntoId;
use result::{Result, Map};
use error::Error;
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
            Tag::ArrayPattern => {
                let list = self.extract_array("elements")?;
                let mut objs = list.map(|v| {
                    match v.is_null() {
                        true => Ok(None),
                        false => Ok(Some(v.into_object().map_err(Error::Json)?))
                    }
                })?;

                let mut rest = None;
                if let Some(Some(mut last)) = objs.pop() {
                    if last.tag()? == Tag::RestElement {
                        rest = Some(RestPatt {
                            location: None,
                            patt: last.extract_patt("argument")?
                        });
                    } else {
                        objs.push(Some(last));
                    }
                }

                let patt_elements = objs.map(|e| {
                    match e {
                        None => Ok(None),
                        Some(o) => Ok(Some(o.into_patt()?))
                    }
                })?;

                let rest = rest.map(Box::new);

                Ok(Patt::Compound(CompoundPatt::Arr(None, patt_elements, rest)))
            }
            _ => self.into_id().map(|id| id.into_patt())
        }
    }
}
