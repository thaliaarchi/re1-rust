// Copyright 2023 Thalia Archibald. All Rights Reserved.
// Copyright 2007-2009 Russ Cox. All Rights Reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::str::CharIndices;

use crate::{Inst, Prog};

pub fn match_recursive(prog: &mut Prog, mut s: CharIndices<'_>, saved: &mut [usize]) -> bool {
    let inst = match prog.inst() {
        Some(inst) => inst,
        None => return false,
    };
    match *inst {
        Inst::Char(ch) => {
            if let Some((_, ch1)) = s.next() {
                ch == ch1 && match_recursive(prog.next(), s, saved)
            } else {
                false
            }
        }
        Inst::Any => {
            if s.next().is_some() {
                match_recursive(prog.next(), s, saved)
            } else {
                false
            }
        }
        Inst::Match => true,
        Inst::Jmp(x) => {
            prog.pc = x;
            match_recursive(prog, s, saved)
        }
        Inst::Split(x, y) => {
            if match_recursive(prog.jump(x), s.clone(), saved) {
                return true;
            }
            match_recursive(prog.jump(y), s, saved)
        }
        Inst::Save(n) => {
            prog.next();
            if n >= saved.len() {
                return match_recursive(prog, s, saved);
            }
            let old = saved[n];
            saved[n] = s.offset();
            if match_recursive(prog, s, saved) {
                return true;
            }
            saved[n] = old;
            false
        }
    }
}
