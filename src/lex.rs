// Copyright 2023 Thalia Archibald. All Rights Reserved.
// Copyright 2007-2009 Russ Cox. All Rights Reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::convert::Infallible;
use std::fmt::{self, Display, Formatter, Write};
use std::str::CharIndices;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Alt,
    Star,
    Plus,
    Quest,
    ParenL,
    ParenR,
    Colon,
    Dot,
    Char(char),
}

#[derive(Clone, Debug)]
pub struct Lexer<'a> {
    chars: CharIndices<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(s: &'a str) -> Self {
        Lexer {
            chars: s.char_indices(),
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<(usize, Token, usize), Infallible>;

    fn next(&mut self) -> Option<Self::Item> {
        let (i, ch) = self.chars.next()?;
        let tok = match ch {
            '|' => Token::Alt,
            '*' => Token::Star,
            '+' => Token::Plus,
            '?' => Token::Quest,
            '(' => Token::ParenL,
            ')' => Token::ParenR,
            ':' => Token::Colon,
            '.' => Token::Dot,
            ch => Token::Char(ch),
        };
        Some(Ok((i, tok, i + ch.len_utf8())))
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_char(match self {
            Token::Alt => '|',
            Token::Star => '*',
            Token::Plus => '+',
            Token::Quest => '?',
            Token::ParenL => '(',
            Token::ParenR => ')',
            Token::Colon => ':',
            Token::Dot => '.',
            Token::Char(ch) => *ch,
        })
    }
}
