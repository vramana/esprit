use easter::fun::{Fun, FunctionKind};
use easter::stmt::empty_script;
use unjson::ty::Object;
use unjson::ExtractField;
// use serde_json::to_string_pretty;

use result::Result;
use node::ExtractNode;
use tag::{Tag, TagOf};
use expr::IntoExpr;

pub trait IntoFun {
    fn into_fun(self, FunctionKind) -> Result<Fun>;
    fn into_arrow_function(self) -> Result<Fun>;
}

impl IntoFun for Object {
    fn into_fun(mut self, kind: FunctionKind) -> Result<Fun> {
        // println!("{}", to_string_pretty(&self).unwrap());
        let is_generator = self.extract_bool_opt("generator")?;
        let kind = match (kind, is_generator) {
            (FunctionKind::Named(id), Some(true)) => FunctionKind::Generator(id),
            (FunctionKind::Anonymous, Some(true)) => FunctionKind::AnonymousGenerator,
            (r @ _, _) => r
        };

        let params = self.extract_params("params")?;
        let mut obj = self.extract_object("body")?;
        let body = obj.extract_script("body")?;
        Ok(Fun {
            location: None,
            kind: kind,
            params: params,
            body: body,
            body_expr: None
        })
    }

    fn into_arrow_function(mut self) -> Result<Fun> {
        let params = self.extract_params("params")?;
        let mut obj = self.extract_object("body")?;
        match obj.tag()? {
            Tag::BlockStatement => {
                let body = obj.extract_script("body")?;
                Ok(Fun {
                    location: None,
                    kind: FunctionKind::Arrow,
                    params: params,
                    body: body,
                    body_expr: None
                })
            }
            _ => {
                let expr = obj.into_expr()?;
                Ok(Fun {
                    location: None,
                    kind: FunctionKind::Arrow,
                    params: params,
                    body: empty_script(),
                    body_expr: Some(Box::new(expr))
                })
            }
        }
    }
}
