// Copyright 2023 Thalia Archibald. All Rights Reserved.
// Copyright 2007-2009 Russ Cox. All Rights Reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Regexp {
    Alt(Box<Regexp>, Box<Regexp>),
    Cat(Box<Regexp>, Box<Regexp>),
    Lit(char),
    Dot,
    Paren(u32, Box<Regexp>),
    Quest(/*greedy*/ bool, Box<Regexp>),
    Star(/*greedy*/ bool, Box<Regexp>),
    Plus(/*greedy*/ bool, Box<Regexp>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inst {
    Char(char),
    Match,
    Jmp(usize),
    Split(usize, usize),
    Any,
    Save(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prog {
    pub insts: Vec<Inst>,
}

impl Display for Regexp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Regexp::Alt(left, right) => write!(f, "Alt({left}, {right})"),
            Regexp::Cat(left, right) => write!(f, "Cat({left}, {right})"),
            Regexp::Lit(ch) => write!(f, "Lit({ch})"),
            Regexp::Dot => write!(f, "Dot"),
            Regexp::Paren(n, inner) => write!(f, "Paren({n}, {inner})"),
            Regexp::Quest(true, inner) => write!(f, "Quest({inner})"),
            Regexp::Quest(false, inner) => write!(f, "NgQuest({inner})"),
            Regexp::Star(true, inner) => write!(f, "Star({inner})"),
            Regexp::Star(false, inner) => write!(f, "NgStar({inner})"),
            Regexp::Plus(true, inner) => write!(f, "Plus({inner})"),
            Regexp::Plus(false, inner) => write!(f, "NgPlus({inner})"),
        }
    }
}

impl Display for Inst {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Inst::Char(ch) => write!(f, "char {ch}"),
            Inst::Match => write!(f, "match"),
            Inst::Jmp(x) => write!(f, "jmp {x}"),
            Inst::Split(x, y) => write!(f, "split {x}, {y}"),
            Inst::Any => write!(f, "any"),
            Inst::Save(n) => write!(f, "save {n}"),
        }
    }
}

impl Display for Prog {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (pc, inst) in self.insts.iter().enumerate() {
            writeln!(f, "{pc:2}. {inst}")?;
        }
        Ok(())
    }
}
