// Copyright 2023 Thalia Archibald. All Rights Reserved.
// Copyright 2007-2009 Russ Cox. All Rights Reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::convert::Infallible;
use std::fmt::{self, Display, Formatter};

use lalrpop_util::ParseError;

use crate::lex::{Lexer, Token};
use crate::parse::RegexpParser;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Regexp {
    Alt(Box<Regexp>, Box<Regexp>),
    Cat(Box<Regexp>, Box<Regexp>),
    Lit(char),
    Dot,
    Paren(usize, Box<Regexp>),
    Quest(/*greedy*/ bool, Box<Regexp>),
    Star(/*greedy*/ bool, Box<Regexp>),
    Plus(/*greedy*/ bool, Box<Regexp>),
}

impl Regexp {
    pub fn parse(s: &str) -> Result<Box<Regexp>, ParseError<usize, Token, Infallible>> {
        RegexpParser::new().parse(Lexer::new(s))
    }

    pub fn parse_wrapped(s: &str) -> Result<Box<Regexp>, ParseError<usize, Token, Infallible>> {
        let re = Regexp::parse(s)?;
        let paren = Box::new(Regexp::Paren(0, re));
        let dot_star = Box::new(Regexp::Star(false, Box::new(Regexp::Dot)));
        Ok(Box::new(Regexp::Cat(dot_star, paren)))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inst {
    Char(char),
    Match,
    Jmp(usize),
    Split(usize, usize),
    Any,
    Save(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prog {
    pub(crate) insts: Vec<Inst>,
}

impl Prog {
    #[inline]
    pub fn insts(&self) -> &[Inst] {
        &self.insts
    }

    #[inline]
    pub fn inst(&self, pc: usize) -> Option<&Inst> {
        self.insts.get(pc)
    }

    pub fn nsaved(&self) -> usize {
        let mut count = 0;
        for inst in &self.insts {
            if let Inst::Save(n) = inst {
                count = count.max(n + 1);
            }
        }
        count
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VM<'i, 's> {
    insts: &'i [Inst],
    pc: usize,
    s: &'s str,
    offset: usize,
    debug: bool,
}

impl<'i, 's> VM<'i, 's> {
    #[inline]
    pub fn new(prog: &'i Prog, s: &'s str, debug: bool) -> Self {
        VM {
            insts: &prog.insts,
            pc: 0,
            s,
            offset: 0,
            debug,
        }
    }

    pub fn next_inst(&mut self) -> Option<&Inst> {
        let inst = self.insts.get(self.pc);
        if self.debug {
            println!("{self}");
        }
        if inst.is_some() {
            self.pc += 1;
        }
        inst
    }

    #[inline]
    pub fn next_char(&mut self) -> Option<char> {
        let mut chars = self.s[self.offset..].chars();
        let ch = chars.next();
        self.offset = self.s.len() - chars.as_str().len();
        ch
    }

    #[inline]
    pub fn inst(&self) -> Option<&Inst> {
        self.insts.get(self.pc)
    }

    #[inline]
    pub fn pc(&self) -> usize {
        self.pc
    }

    #[inline]
    pub fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    pub fn reset(&mut self) {
        self.pc = 0;
        self.offset = 0;
    }
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

impl Display for VM<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[offset {}] {:2}. ", self.offset, self.pc)?;
        match self.inst() {
            Some(inst) => write!(f, "{inst}"),
            None => write!(f, "-"),
        }
    }
}
