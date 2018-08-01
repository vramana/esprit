//! A JavaScript parsing library.
//!
//! Esprit currently parses all of ES5 and bits of ES6. The goal
//! is to support all of ES6.
//!
//! Currently the parser is hard-coded to produce the Easter AST
//! data structures. Eventually it should be abstracted to support
//! pluggable builders.

extern crate serde;
extern crate serde_json;
extern crate tristate;
extern crate estree;
extern crate unjson;
extern crate easter;
extern crate joker;

pub mod error;
pub mod result;
mod context;
mod tokens;
mod atom;
mod track;
mod parser;
mod state;
mod expr;
mod stack;

// type Parser<I: Iterator<Item=char>> = parser::Parser<I>;

use easter::stmt::{Script, Module};
use parser::Parser;
use result::Result;

pub use parser::Program;

pub fn script(s: &str) -> Result<Script> {
    Parser::from_chars(s.chars()).script(false)
}

pub fn strict(s: &str) -> Result<Script> {
    Parser::from_chars(s.chars()).script(true)
}

pub fn module(s: &str) -> Result<Module> {
    Parser::from_chars(s.chars()).module()
}

pub fn program(s: &str) -> Result<Program> {
    Parser::from_chars(s.chars()).program()
}
