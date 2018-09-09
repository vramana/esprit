use std::fmt;
use std::fmt::{Debug, Formatter};
use joker::track::{TrackingRef, TrackingMut, Span, Untrack};
use joker::token::{NumberLiteral, StringLiteral, RegExpLiteral};

use obj::{DotKey, Prop, PropVal};
use fun::Fun;
use punc::{Unop, Binop, Assop, Logop};
use id::Id;
use patt::{Patt, RestPatt, CompoundPatt, PropPatt};
use cover;

#[derive(PartialEq, Debug, Clone, TrackingRef, TrackingMut, Untrack)]
pub enum Assign {
    Expr(Expr),
    Pattern(Patt<Id>)
}

#[derive(PartialEq, Debug, Clone, TrackingRef, TrackingMut, Untrack)]
pub enum ExprListItem {
    Expr(Expr),
    Spread(Option<Span>, Expr)
}

#[derive(Clone, TrackingRef, TrackingMut, Untrack)]
pub enum Expr {
    This(Option<Span>),
    Id(Id),
    Arr(Option<Span>, Vec<Option<ExprListItem>>),
    Obj(Option<Span>, Vec<Prop>),
    Fun(Fun),
    Seq(Option<Span>, Vec<Expr>),
    Unop(Option<Span>, Unop, Box<Expr>),
    Binop(Option<Span>, Binop, Box<Expr>, Box<Expr>),
    Logop(Option<Span>, Logop, Box<Expr>, Box<Expr>),
    PreInc(Option<Span>, Box<Expr>),
    PostInc(Option<Span>, Box<Expr>),
    PreDec(Option<Span>, Box<Expr>),
    PostDec(Option<Span>, Box<Expr>),
    Assign(Option<Span>, Box<Assign>, Box<Expr>),
    BinAssign(Option<Span>, Assop, Box<Expr>, Box<Expr>),
    Cond(Option<Span>, Box<Expr>, Box<Expr>, Box<Expr>),
    Call(Option<Span>, Box<Expr>, Vec<ExprListItem>),
    New(Option<Span>, Box<Expr>, Option<Vec<ExprListItem>>),
    Dot(Option<Span>, Box<Expr>, DotKey),
    Brack(Option<Span>, Box<Expr>, Box<Expr>),
    NewTarget(Option<Span>),
    True(Option<Span>),
    False(Option<Span>),
    Null(Option<Span>),
    Number(Option<Span>, NumberLiteral),
    RegExp(Option<Span>, RegExpLiteral),
    String(Option<Span>, StringLiteral)
}

// TODO May be move this back into cover. Or remove it altogether and bring Error here.
impl Expr {
    pub fn is_assignable(&self) -> bool  {
        match self {
            Expr::Dot(_, _, _)
          | Expr::Brack(_, _, _)
          | Expr::Id(_) => true,
          _ => false
        }
    }

    pub fn into_assignable(self) -> Result<Expr, cover::Error>  {
        match self {
            Expr::Dot(_, _, _)
          | Expr::Brack(_, _, _)
          | Expr::Id(_) => Ok(self),
          _ => Err(cover::Error::InvalidAssignTarget(*self.tracking_ref()))
        }
    }

    pub fn into_simple_or_compound_pattern(self) -> Result<Patt<Expr>, cover::Error> {
        match self {
            Expr::Obj(location, props) => {
                let mut prop_patts = Vec::with_capacity(props.len());
                for prop in props {
                    prop_patts.push(prop.into_assign_prop()?);
                }
                Ok(Patt::Compound(CompoundPatt::Obj(location, prop_patts)))
            }
            Expr::Arr(location, mut exprs) => {
                let mut patts = Vec::with_capacity(exprs.len());
                let mut rest = None;
                if let Some(last) = exprs.pop() {
                    if let Some(ExprListItem::Spread(None, expr)) = last {
                        rest = Some(Box::new(RestPatt {
                            location: None,
                            patt: expr.into_simple_or_compound_pattern()?
                        }));
                    } else {
                        exprs.push(last);
                    }
                }
                for expr in exprs {
                    patts.push(match expr {
                        Some(ExprListItem::Expr(expr)) => Some(expr.into_simple_or_compound_pattern()?),
                        Some(ExprListItem::Spread(loc, _)) => { return Err(cover::Error::InvalidAssignTarget(loc)); }
                        None => None
                    });
                }
                Ok(Patt::Compound(CompoundPatt::Arr(location, patts, rest)))
            }
            _ => { return self.into_assignable().map(Patt::Simple); }
        }
    }
}

// TODO May be move this back into cover. Or remove it altogether and bring Error here.
pub trait IntoAssignProp {
    fn into_assign_prop(self) -> Result<PropPatt<Expr>, cover::Error>;
}

impl IntoAssignProp for Prop {
    fn into_assign_prop(self) -> Result<PropPatt<Expr>, cover::Error> {
        let location = *self.tracking_ref();
        Ok(match self {
            Prop::Regular(location, key, PropVal::Init(expr)) => {
                PropPatt::Regular(location, key, expr.into_simple_or_compound_pattern()?)
            }
            Prop::Shorthand(id) => {
                PropPatt::Shorthand(None, id, None)
            }
            _ => { return Err(cover::Error::InvalidPropPatt(location)); }
        })
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        if *self.tracking_ref() != *other.tracking_ref() {
            return false;
        }
        match (self, other) {
            (&Expr::This(_),                      &Expr::This(_))                      => true,
            (&Expr::Id(ref id_l),                 &Expr::Id(ref id_r))                 => id_l == id_r,
            (&Expr::Arr(_, ref elts_l),           &Expr::Arr(_, ref elts_r))           => elts_l == elts_r,
            (&Expr::Obj(_, ref props_l),          &Expr::Obj(_, ref props_r))          => props_l == props_r,
            (&Expr::Fun(ref fun_l),               &Expr::Fun(ref fun_r))               => fun_l == fun_r,
            (&Expr::Seq(_, ref exprs_l),          &Expr::Seq(_, ref exprs_r))          => exprs_l == exprs_r,
            (&Expr::Unop(_, ref op_l, ref arg_l), &Expr::Unop(_, ref op_r, ref arg_r)) => (op_l, arg_l) == (op_r, arg_r),
            (&Expr::Binop(_, ref op_l, ref arg1_l, ref arg2_l),
             &Expr::Binop(_, ref op_r, ref arg1_r, ref arg2_r))                        => (op_l, arg1_l, arg2_l) == (op_r, arg1_r, arg2_r),
            (&Expr::Logop(_, ref op_l, ref arg1_l, ref arg2_l),
             &Expr::Logop(_, ref op_r, ref arg1_r, ref arg2_r))                        => (op_l, arg1_l, arg2_l) == (op_r, arg1_r, arg2_r),
            (&Expr::PreInc(_, ref arg_l),         &Expr::PreInc(_, ref arg_r))
          | (&Expr::PostInc(_, ref arg_l),        &Expr::PostInc(_, ref arg_r))
          | (&Expr::PreDec(_, ref arg_l),         &Expr::PreDec(_, ref arg_r))
          | (&Expr::PostDec(_, ref arg_l),        &Expr::PostDec(_, ref arg_r))        => arg_l == arg_r,
            (&Expr::Assign(_, ref patt_l, ref arg_l),
             &Expr::Assign(_, ref patt_r, ref arg_r))                                  => (patt_l, arg_l) == (patt_r, arg_r),
            (&Expr::BinAssign(_, ref op_l, ref patt_l, ref arg_l),
             &Expr::BinAssign(_, ref op_r, ref patt_r, ref arg_r))                     => (op_l, patt_l, arg_l) == (op_r, patt_r, arg_r),
            (&Expr::Cond(_, ref test_l, ref cons_l, ref alt_l),
             &Expr::Cond(_, ref test_r, ref cons_r, ref alt_r))                        => (test_l, cons_l, alt_l) == (test_r, cons_r, alt_r),
            (&Expr::Call(_, ref callee_l, ref args_l),
             &Expr::Call(_, ref callee_r, ref args_r))                                 => (callee_l, args_l) == (callee_r, args_r),
            (&Expr::New(_, ref callee_l, None),   &Expr::New(_, ref callee_r, None))   => callee_l == callee_r,
            (&Expr::New(_, ref callee_l, None),   &Expr::New(_, ref callee_r, Some(ref args)))
          | (&Expr::New(_, ref callee_l, Some(ref args)),
             &Expr::New(_, ref callee_r, None))                                        => (callee_l == callee_r) && args.is_empty(),
            (&Expr::New(_, ref callee_l, Some(ref args_l)),
             &Expr::New(_, ref callee_r, Some(ref args_r)))                            => (callee_l, args_l) == (callee_r, args_r),
            (&Expr::Dot(_, ref obj_l, ref key_l), &Expr::Dot(_, ref obj_r, ref key_r)) => (obj_l, key_l) == (obj_r, key_r),
            (&Expr::Brack(_, ref obj_l, ref prop_l),
             &Expr::Brack(_, ref obj_r, ref prop_r))                                   => (obj_l, prop_l) == (obj_r, prop_r),
            (&Expr::NewTarget(_),          &Expr::NewTarget(_))                        => true,
            (&Expr::True(_),               &Expr::True(_))                             => true,
            (&Expr::False(_),              &Expr::False(_))                            => true,
            (&Expr::Null(_),               &Expr::Null(_))                             => true,
            (&Expr::Number(_, ref lit_l),  &Expr::Number(_, ref lit_r))                => lit_l == lit_r,
            (&Expr::RegExp(_, ref lit_l),  &Expr::RegExp(_, ref lit_r))                => lit_l == lit_r,
            (&Expr::String(_, ref lit_l),  &Expr::String(_, ref lit_r))                => lit_l == lit_r,
            _ => false
        }
    }
}

impl Debug for Expr {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            &Expr::This(_)                                   => fmt.write_str("This"),
            &Expr::Id(ref id)                                => fmt.debug_tuple("Id").field(id).finish(),
            &Expr::Arr(_, ref elts)                          => fmt.debug_tuple("Arr").field(elts).finish(),
            &Expr::Obj(_, ref props)                         => fmt.debug_tuple("Obj").field(props).finish(),
            &Expr::Fun(ref fun)                              => fmt.debug_tuple("Fun").field(fun).finish(),
            &Expr::Seq(_, ref exprs)                         => fmt.debug_tuple("Seq").field(exprs).finish(),
            &Expr::Unop(_, ref op, ref arg)                  => fmt.debug_tuple("Unop").field(op).field(arg).finish(),
            &Expr::Binop(_, ref op, ref left, ref right)     => fmt.debug_tuple("Binop").field(op).field(left).field(right).finish(),
            &Expr::Logop(_, ref op, ref left, ref right)     => fmt.debug_tuple("Logop").field(op).field(left).field(right).finish(),
            &Expr::PreInc(_, ref arg)                        => fmt.debug_tuple("PreInc").field(arg).finish(),
            &Expr::PostInc(_, ref arg)                       => fmt.debug_tuple("PostInc").field(arg).finish(),
            &Expr::PreDec(_, ref arg)                        => fmt.debug_tuple("PreDec").field(arg).finish(),
            &Expr::PostDec(_, ref arg)                       => fmt.debug_tuple("PostDec").field(arg).finish(),
            &Expr::Assign(_, ref left, ref right)            => fmt.debug_tuple("Assign").field(left).field(right).finish(),
            &Expr::BinAssign(_, ref op, ref left, ref right) => fmt.debug_tuple("BinAssign").field(op).field(left).field(right).finish(),
            &Expr::Cond(_, ref test, ref cons, ref alt)      => fmt.debug_tuple("Cond").field(test).field(cons).field(alt).finish(),
            &Expr::Call(_, ref callee, ref args)             => fmt.debug_tuple("Call").field(callee).field(args).finish(),
            &Expr::New(_, ref ctor, None) => {
                let args: Vec<Expr> = vec![];
                fmt.debug_tuple("New").field(ctor).field(&args).finish()
            }
            &Expr::New(_, ref ctor, Some(ref args))          => fmt.debug_tuple("New").field(ctor).field(args).finish(),
            &Expr::Dot(_, ref expr, ref key)                 => fmt.debug_tuple("Dot").field(expr).field(key).finish(),
            &Expr::Brack(_, ref expr, ref prop)              => fmt.debug_tuple("Brack").field(expr).field(prop).finish(),
            &Expr::NewTarget(_)                              => fmt.write_str("NewTarget"),
            &Expr::True(_)                                   => fmt.write_str("True"),
            &Expr::False(_)                                  => fmt.write_str("False"),
            &Expr::Null(_)                                   => fmt.write_str("Null"),
            &Expr::Number(_, ref lit)                        => fmt.debug_tuple("Number").field(lit).finish(),
            &Expr::RegExp(_, ref lit)                        => fmt.debug_tuple("RegExp").field(lit).finish(),
            &Expr::String(_, ref lit)                        => fmt.debug_tuple("String").field(lit).finish()
        }
    }
}
