use std::fmt;
use std::fmt::{Debug, Formatter};
use track::{Posn, Span, Untrack};
use word::{Name, Reserved};

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub location: Span,
    pub newline: bool, // was there a newline between the preceding token and this one?
    pub value: TokenData,
}

impl Token {
    pub fn new(start: Posn, end: Posn, value: TokenData) -> Token {
        Token {
            location: Span {
                start: start,
                end: end,
            },
            newline: false,
            value: value,
        }
    }

    pub fn len(&self) -> usize {
        (self.location.end.column - self.location.start.column) as usize
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenData {
    Reserved(Reserved),

    // 11.7 Punctuators
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBrack,
    RBrack,
    Dot,
    Ellipsis,
    Semi,
    Comma,
    LAngle,
    RAngle,
    LEq,
    GEq,
    Eq,
    NEq,
    StrictEq,
    StrictNEq,
    Plus,
    Minus,
    Star,
    Mod,
    Slash,
    Inc,
    Dec,
    LShift,
    RShift,
    URShift,
    BitAnd,
    BitOr,
    BitXor,
    Bang,
    Tilde,
    LogicalAnd,
    LogicalOr,
    Question,
    Colon,
    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    ModAssign,
    LShiftAssign,
    RShiftAssign,
    URShiftAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    Arrow,

    Number(NumberLiteral),
    String(StringLiteral),
    RegExp(RegExpLiteral),

    Identifier(Name),

    EOF,
}

impl TokenData {
    pub fn is_string(&self) -> bool {
        match *self {
            TokenData::String(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct RegExpLiteral {
    pub pattern: String,
    pub flags: Vec<char>,
}

impl Untrack for RegExpLiteral {
    fn untrack(&mut self) {}
}

// TODO move to impl RegExpLiteral

trait CharsEx {
    fn alphabetize(&self) -> Vec<char>;
}

impl CharsEx for Vec<char> {
    fn alphabetize(&self) -> Vec<char> {
        let mut x: Vec<char> = self.to_vec();
        x.sort();
        x
    }
}

impl Debug for RegExpLiteral {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("RegExpLiteral")
            .field("pattern", &self.pattern)
            .field("flags", &self.flags.alphabetize())
            .finish()
    }
}

impl PartialEq for RegExpLiteral {
    fn eq(&self, other: &Self) -> bool {
        (self.pattern == other.pattern) && (self.flags.alphabetize() == other.flags.alphabetize())
    }
}

#[derive(Clone)]
pub struct StringLiteral {
    pub source: Option<String>,
    pub value: String,
}

impl Untrack for StringLiteral {
    fn untrack(&mut self) {}
}

impl Debug for StringLiteral {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("StringLiteral")
            .field("value", &self.value)
            .finish()
    }
}

impl PartialEq for StringLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

#[derive(Clone)]
pub struct NumberLiteral {
    pub source: Option<NumberSource>,
    pub value: f64,
}

impl Untrack for NumberLiteral {
    fn untrack(&mut self) {}
}

impl Debug for NumberLiteral {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("NumberLiteral")
            .field("value", &self.value)
            .finish()
    }
}

impl PartialEq for NumberLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum NumberSource {
    DecimalInt(String, Option<Exp>),
    RadixInt(Radix, String),
    Float(Option<String>, Option<String>, Option<Exp>),
}

fn format_sign(sign: &Option<Sign>) -> String {
    match *sign {
        Some(Sign::Minus) => "-",
        _ => "",
    }.to_string()
}

fn format_int(src: &Option<String>) -> String {
    match *src {
        None => "".to_string(),
        Some(ref s) => s.to_string(),
    }
}

impl NumberSource {
    pub fn value(&self) -> f64 {
        match *self {
            NumberSource::DecimalInt(ref mantissa, None) => mantissa.parse().unwrap(),
            NumberSource::DecimalInt(
                ref mantissa,
                Some(Exp {
                    ref sign,
                    ref value,
                    ..
                }),
            ) => format!("{}e{}{}", mantissa, format_sign(sign), value)
                .parse()
                .unwrap(),
            NumberSource::RadixInt(ref radix, ref src) => {
                let radix = radix.value();
                let mult = radix as f64;

                src.chars()
                    .map(|c| c.to_digit(radix).unwrap() as f64)
                    .fold(0f64, |res, digit| res.mul_add(mult, digit))
            }
            NumberSource::Float(ref ip, ref fp, None) => {
                format!("{}.{}", format_int(ip), format_int(fp))
                    .parse()
                    .unwrap()
            }
            NumberSource::Float(
                ref ip,
                ref fp,
                Some(Exp {
                    ref sign,
                    ref value,
                    ..
                }),
            ) => format!(
                "{}.{}e{}{}",
                format_int(ip),
                format_int(fp),
                format_sign(sign),
                value
            ).parse()
                .unwrap(),
        }
    }

    pub fn into_token_data(self) -> TokenData {
        let value = self.value();
        TokenData::Number(NumberLiteral {
            source: Some(self),
            value: value,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Exp {
    pub e: CharCase,
    pub sign: Option<Sign>,
    pub value: String,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Radix {
    Bin(CharCase),
    Oct(Option<CharCase>),
    Hex(CharCase),
}

impl Radix {
    pub fn value(&self) -> u32 {
        match *self {
            Radix::Bin(_) => 2,
            Radix::Oct(_) => 8,
            Radix::Hex(_) => 16,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CharCase {
    LowerCase,
    UpperCase,
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[cfg_attr(test, derive(Deserialize))]
pub enum Sign {
    #[cfg_attr(test, serde(rename = "+"))]
    Plus,

    #[cfg_attr(test, serde(rename = "-"))]
    Minus,
}
