use std::fmt;
use std::fmt::{Display, Formatter};
use serde_json::map::Map;
use serde_json::value::Value;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Ty {
    Null,
    Boolean,
    String,
    Number,
    Object,
    Array
}

pub type Array = Vec<Value>;
pub type Object = Map<String, Value>;

pub trait TyOf {
    fn ty(&self) -> Ty;
}

impl TyOf for Value {
    fn ty(&self) -> Ty {
        match *self {
            Value::Null      => Ty::Null,
            Value::String(_) => Ty::String,
            Value::Object(_) => Ty::Object,
            Value::Number(_) => Ty::Number,
            Value::Array(_)  => Ty::Array,
            Value::Bool(_)   => Ty::Boolean
        }
    }
}

impl Display for Ty {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.write_str(match *self {
            Ty::Null    => "null",
            Ty::String  => "string",
            Ty::Object  => "object",
            Ty::Number  => "number",
            Ty::Array   => "array",
            Ty::Boolean => "boolean"
        })
    }
}
