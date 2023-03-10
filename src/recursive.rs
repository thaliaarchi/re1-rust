// Copyright 2023 Thalia Archibald. All Rights Reserved.
// Copyright 2007-2009 Russ Cox. All Rights Reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use crate::{Inst, Sub, VM};

impl VM<'_, '_> {
    pub fn match_recursive(&mut self, sub: &mut Sub) -> bool {
        let inst = match self.next_inst() {
            Some(inst) => inst,
            None => return false,
        };
        match *inst {
            Inst::Char(ch) => {
                if let Some(ch1) = self.next_char() {
                    ch == ch1 && self.match_recursive(sub)
                } else {
                    false
                }
            }
            Inst::Any => {
                if self.next_char().is_some() {
                    self.match_recursive(sub)
                } else {
                    false
                }
            }
            Inst::Match => true,
            Inst::Jmp(x) => {
                self.pc = x;
                self.match_recursive(sub)
            }
            Inst::Split(x, y) => {
                self.pc = x;
                if self.clone().match_recursive(sub) {
                    return true;
                }
                self.pc = y;
                self.match_recursive(sub)
            }
            Inst::Save(n) => {
                if n >= sub.len() {
                    return self.match_recursive(sub);
                }
                let old = sub.get(n);
                sub.set(n, self.offset);
                if self.match_recursive(sub) {
                    return true;
                }
                sub.set(n, old);
                false
            }
        }
    }

    pub fn match_recursive_loop(&mut self, sub: &mut Sub) -> bool {
        loop {
            let inst = match self.next_inst() {
                Some(inst) => inst,
                None => return false,
            };
            match *inst {
                Inst::Char(ch) => {
                    if self.next_char() != Some(ch) {
                        return false;
                    }
                }
                Inst::Any => {
                    if self.next_char().is_none() {
                        return false;
                    }
                }
                Inst::Match => return true,
                Inst::Jmp(x) => self.pc = x,
                Inst::Split(x, y) => {
                    self.pc = x;
                    if self.clone().match_recursive_loop(sub) {
                        return true;
                    }
                    self.pc = y;
                }
                Inst::Save(n) => {
                    if n >= sub.len() {
                        continue;
                    }
                    let old = sub.get(n);
                    sub.set(n, self.offset);
                    if self.match_recursive_loop(sub) {
                        return true;
                    }
                    sub.set(n, old);
                    return false;
                }
            }
        }
    }
}
