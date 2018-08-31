use joker::track::*;
use joker::token::{StringLiteral, NumberLiteral};

use id::Id;
use expr::Expr;
use stmt::Script;
use patt::Patt;
use fun::Fun;

#[derive(Debug, PartialEq, Clone, TrackingRef, TrackingMut)]
pub struct DotKey {
    pub location: Option<Span>,
    pub value: String
}

impl Untrack for DotKey {
    fn untrack(&mut self) { self.location = None; }
}

#[derive(Debug, PartialEq, Clone, TrackingRef, TrackingMut, Untrack)]
pub enum Prop {
    Regular(Option<Span>, PropKey, PropVal),
    Method(PropKey, Fun),
    Shorthand(Id)
}

#[derive(Debug, PartialEq, Clone, TrackingRef, TrackingMut)]
pub enum PropKey {
    Id(Option<Span>, String),
    String(Option<Span>, StringLiteral),
    Number(Option<Span>, NumberLiteral)
}

impl Untrack for PropKey {
    fn untrack(&mut self) {
        *self.tracking_mut() = None;
    }
}

#[derive(Debug, PartialEq, Clone, TrackingRef, TrackingMut, Untrack)]
pub enum PropVal {
    Init(Expr),
    Get(Option<Span>, Script),
    Set(Option<Span>, Patt<Id>, Script)
}
