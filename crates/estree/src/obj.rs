use easter::fun::{FunctionKind};
use easter::obj::{Prop, PropKey, PropVal};
use easter::expr::Expr;
use unjson::ty::{Object, Ty};
use unjson::ExtractField;

use tag::{Tag, TagOf};
use id::IntoId;
use result::Result;
use error::{type_error, array_error};
use node::ExtractNode;
use expr::IntoExpr;
use fun::IntoFun;

pub trait IntoObj {
    fn into_prop(self) -> Result<Prop>;
    fn into_prop_key(self, Option<bool>) -> Result<PropKey>;
    fn computed(&self) -> Option<bool>;
}

impl IntoObj for Object {
    fn into_prop(mut self) -> Result<Prop> {
        let key = self.extract_object("key")?;
        let computed = self.computed();
        let mut val = self.extract_object("value")?;
        let kind = self.extract_string("kind")?;
        let val = match &kind[..] {
            "init" => {
                if self.extract_bool("method")? {
                    let fun = val.into_fun(FunctionKind::Anonymous)?;
                    return Ok(Prop::Method(key.into_prop_key(computed)?, fun))
                } else if self.extract_bool("shorthand")? {
                    return Ok(Prop::Shorthand(key.into_id()?));
                } else {
                    PropVal::Init(val.into_expr()?)
                }
            },
            "get" => PropVal::Get(None, val.extract_object("body")?.extract_script("body")?),
            "set" => {
                let fun = val.into_fun(FunctionKind::Anonymous)?;
                let params = fun.params.list;
                if params.len() != 1 {
                    return array_error(1, params.len());
                }
                let param = params.into_iter().next().unwrap();
                PropVal::Set(None, param, fun.body)
            }
            _ => { return type_error("'init', 'get', or 'set'", Ty::String); }
        };
        Ok(Prop::Regular(None, key.into_prop_key(computed)?, val))
    }

    fn into_prop_key(self, computed: Option<bool>) -> Result<PropKey> {
        match self.tag()? {
            Tag::Identifier => {
                let id = self.into_id()?;
                match computed {
                    Some(true) => Ok(PropKey::Computed(None, Expr::Id(id))),
                    Some(false) => Ok(PropKey::Id(None, id.name.into_string())),
                    _ => Ok(PropKey::Id(None, id.name.into_string()))
                }
            }
            Tag::Literal => {
                match self.into_lit()? {
                    Expr::Number(_, lit) => Ok(PropKey::Number(None, lit)),
                    Expr::String(_, lit) => Ok(PropKey::String(None, lit)),
                    _ => type_error("literal is neither number literal nor string literal", Ty::Object)
                }
            }
            _ => {
                match computed {
                    Some(true) => Ok(PropKey::Computed(None, self.into_expr()?)),
                    _ => type_error("property is neither computed property nor literal", Ty::Object),
                }
            }
        }
    }

    fn computed(&self) -> Option<bool> {
        self.get("computed").and_then(|b| b.as_bool())
    }
}
