use easter::patt::{Patt, PropPatt, RestPatt, CompoundPatt};
use easter::id::{Id, IdExt};
use unjson::ty::Object;
use unjson::{ExtractField, Unjson};

use id::IntoId;
use obj::IntoObj;
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
            Tag::ObjectPattern => {
                let list = self.extract_array("properties")?;
                let mut objs = list.map(|v| Ok(v.into_object().map_err(Error::Json)?))?;

                // let mut rest = None;
                // if let Some(mut last) = objs.pop() {
                //     if last.tag()? == Tag::RestElement {
                //         rest = Some(RestPatt {
                //             location: None,
                //             patt: last.extract_patt("argument")?
                //         });
                //     } else {
                //         objs.push(last));
                //     }
                // }

                let patt_elements = objs.map(|mut e| {
                    match e.extract_bool("shorthand")? {
                        true => Ok(PropPatt::Shorthand(e.extract_id("key")?)),
                        false => {
                            let prop_key = e.extract_object("key")?.into_prop_key()?;
                            let prop_value = e.extract_object("value")?.into_patt()?;
                            Ok(PropPatt::Regular(None, prop_key, prop_value))
                        }
                    }
                })?;

                // let rest = rest.map(Box::new);

                Ok(Patt::Compound(CompoundPatt::Obj(None, patt_elements)))
            }
            _ => self.into_id().map(|id| id.into_patt())
        }
    }
}
