use joker::track::*;

use id::Id;
use obj::PropKey;

#[derive(Debug, PartialEq, Clone, TrackingRef, TrackingMut, Untrack)]
pub struct RestPatt<T> {
    pub location: Option<Span>,
    pub patt: Patt<T>
}

#[derive(Debug, PartialEq, Clone, TrackingRef, TrackingMut, Untrack)]
pub enum CompoundPatt<T> {
    Arr(Option<Span>, Vec<Option<Patt<T>>>, Option<Box<RestPatt<T>>>),
    Obj(Option<Span>, Vec<PropPatt<T>>)
}

#[derive(Debug, PartialEq, Clone, TrackingRef, TrackingMut, Untrack)]
pub enum PropPatt<T> {
    Regular(Option<Span>, PropKey, Patt<T>),
    Shorthand(Id)
}

#[derive(Debug, PartialEq, Clone, Untrack)]
pub enum Patt<T> {
    Simple(T),
    Compound(CompoundPatt<T>)
}

impl<T> Patt<T> {
    pub fn is_simple(&self) -> bool {
        match *self {
            Patt::Simple(_)   => true,
            Patt::Compound(_) => false
        }
    }
}


impl<T: TrackingRef> TrackingRef for Patt<T> {
    fn tracking_ref(&self) -> &Option<Span> {
        match *self {
            Patt::Simple(ref simple) => simple.tracking_ref(),
            Patt::Compound(ref patt) => patt.tracking_ref()
        }
    }
}

impl<T: TrackingMut> TrackingMut for Patt<T> {
    fn tracking_mut(&mut self) -> &mut Option<Span> {
        match *self {
            Patt::Simple(ref mut simple) => simple.tracking_mut(),
            Patt::Compound(ref mut patt) => patt.tracking_mut()
        }
    }
}

