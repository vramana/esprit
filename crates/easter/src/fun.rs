use joker::track::*;

use id::Id;
use patt::{Patt, RestPatt};
use stmt::Script;
use expr::Expr;

#[derive(Debug, PartialEq, Clone, TrackingRef, TrackingMut, Untrack)]
pub struct Params {
    pub location: Option<Span>,
    pub list: Vec<Patt<Id>>,
    pub rest: Option<RestPatt<Id>>
}

// This allows regular function, arrow function & generator
// TODO: missing async and async generator
#[derive(Debug, PartialEq, Clone, TrackingRef, TrackingMut, Untrack)]
pub struct Fun {
    pub location: Option<Span>,
    pub kind: FunctionKind,
    pub params: Params,
    // FIXME What is the more cleaner way to solve this? Arrow function need to store an expression
    // conditionally enum totally does not provide the clean way to do it.
    pub body: Script,
    pub body_expr: Option<Box<Expr>>
}

#[derive(Debug, PartialEq, Clone, Untrack)]
pub enum FunctionKind {
    Named(Id),
    Anonymous,
    Arrow,
    Generator(Id),
    AnonymousGenerator
}

// impl TrackingRef for FunctionKind {
//     fn tracking_ref(&self) -> &Option<Span> {
//         &None
//     }
// }

// impl TrackingMut for FunctionKind {
//     fn tracking_mut(&mut self) -> &mut Option<Span> {
//         None
//     }
// }


// impl Untrack for FunctionKind {
//     fn untrack(&mut self) {
//         *self.tracking_mut() = None;
//     }
// }


// impl TrackingMut<Id> for FunctionKind {
//     fn tracking_mut(&mut self) -> &mut Option<Span> {
//         match *self {
//             Program::Ambiguous(_, ref mut script) => script.tracking_mut(),
//             Program::Module(ref mut module) => module.tracking_mut()
//         }
//     }
// }
