// Copyright 2023 Thalia Archibald. All Rights Reserved.
// Copyright 2007-2009 Russ Cox. All Rights Reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::convert::Infallible;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

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
        let mut re = RegexpParser::new().parse(Lexer::new(s))?;
        re.number_parens(1);
        Ok(re)
    }

    pub fn unanchored(self: Box<Self>) -> Box<Self> {
        let paren = Box::new(Regexp::Paren(0, self));
        let dot_star = Box::new(Regexp::Star(false, Box::new(Regexp::Dot)));
        Box::new(Regexp::Cat(dot_star, paren))
    }

    fn number_parens(&mut self, mut next: usize) -> usize {
        match self {
            Regexp::Alt(left, right) | Regexp::Cat(left, right) => {
                next = left.number_parens(next);
                right.number_parens(next)
            }
            Regexp::Lit(_) | Regexp::Dot => next,
            Regexp::Paren(id, inner) => {
                if *id == usize::MAX {
                    *id = next;
                    next += 1;
                }
                inner.number_parens(next)
            }
            Regexp::Quest(_, inner) | Regexp::Star(_, inner) | Regexp::Plus(_, inner) => {
                inner.number_parens(next)
            }
        }
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

    pub fn nsub(&self) -> usize {
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
    pub insts: &'i [Inst],
    pub pc: usize,
    s: &'s str,
    pub offset: usize,
    pub debug: bool,
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
    pub fn reset(&mut self) {
        self.pc = 0;
        self.offset = 0;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sub {
    sub: Box<[usize]>,
}

impl Sub {
    #[inline]
    pub fn new(nsub: usize) -> Self {
        debug_assert!(nsub % 2 == 0);
        Sub {
            sub: vec![usize::MAX; nsub].into(),
        }
    }

    #[inline]
    pub fn get(&self, n: usize) -> usize {
        self.sub[n]
    }

    #[inline]
    pub fn set(&mut self, n: usize, offset: usize) {
        self.sub[n] = offset;
    }

    #[inline]
    pub fn update(self: Rc<Self>, n: usize, offset: usize) -> Rc<Self> {
        let mut sub = Rc::try_unwrap(self).unwrap_or_else(|rc| (*rc).clone());
        sub.set(n, offset);
        Rc::new(sub)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.sub.len()
    }

    #[inline]
    pub fn reset(&mut self) {
        self.sub.fill(usize::MAX);
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

impl Display for Sub {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut j = self.len();
        while j > 0 && self.sub[j - 1] == usize::MAX {
            j -= 1;
        }
        for i in (0..j).step_by(2) {
            if i != 0 {
                write!(f, " ")?;
            }
            write!(f, "(")?;
            if self.sub[i] == usize::MAX {
                write!(f, "?")?;
            } else {
                write!(f, "{}", self.sub[i])?;
            }
            write!(f, ",")?;
            if self.sub[i + 1] == usize::MAX {
                write!(f, "?")?;
            } else {
                write!(f, "{}", self.sub[i + 1])?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}
