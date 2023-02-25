// Copyright 2023 Thalia Archibald. All Rights Reserved.
// Copyright 2007-2009 Russ Cox. All Rights Reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use crate::{Inst, VM};

impl VM<'_, '_> {
    pub fn match_recursive(&mut self, saved: &mut [usize]) -> bool {
        let inst = match self.next_inst() {
            Some(inst) => inst,
            None => return false,
        };
        match *inst {
            Inst::Char(ch) => {
                if let Some(ch1) = self.next_char() {
                    ch == ch1 && self.match_recursive(saved)
                } else {
                    false
                }
            }
            Inst::Any => {
                if self.next_char().is_some() {
                    self.match_recursive(saved)
                } else {
                    false
                }
            }
            Inst::Match => true,
            Inst::Jmp(x) => {
                self.set_pc(x);
                self.match_recursive(saved)
            }
            Inst::Split(x, y) => {
                self.set_pc(x);
                if self.clone().match_recursive(saved) {
                    return true;
                }
                self.set_pc(y);
                self.match_recursive(saved)
            }
            Inst::Save(n) => {
                if n >= saved.len() {
                    return self.match_recursive(saved);
                }
                let old = saved[n];
                saved[n] = self.offset();
                if self.match_recursive(saved) {
                    return true;
                }
                saved[n] = old;
                false
            }
        }
    }

    pub fn match_recursive_loop(&mut self, saved: &mut [usize]) -> bool {
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
                Inst::Jmp(x) => self.set_pc(x),
                Inst::Split(x, y) => {
                    self.set_pc(x);
                    if self.clone().match_recursive_loop(saved) {
                        return true;
                    }
                    self.set_pc(y);
                }
                Inst::Save(n) => {
                    if n >= saved.len() {
                        continue;
                    }
                    let old = saved[n];
                    saved[n] = self.offset();
                    if self.match_recursive_loop(saved) {
                        return true;
                    }
                    saved[n] = old;
                    return false;
                }
            }
        }
    }
}
