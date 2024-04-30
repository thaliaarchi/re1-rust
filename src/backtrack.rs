// Copyright 2023 Thalia Archibald. All Rights Reserved.
// Copyright 2007-2009 Russ Cox. All Rights Reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::rc::Rc;

use crate::{Inst, Sub, VM};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Thread {
    pc: usize,
    offset: usize,
    sub: Rc<Sub>,
}

const MAX_THREADS: usize = 1000;

impl VM<'_, '_> {
    pub fn match_backtrack(&mut self, sub_out: &mut Sub) -> bool {
        let mut ready = Vec::with_capacity(MAX_THREADS);
        ready.push(Thread {
            pc: 0,
            offset: self.offset,
            sub: Rc::new(Sub::new(sub_out.len())),
        });

        while let Some(thread) = ready.pop() {
            self.pc = thread.pc;
            self.offset = thread.offset;
            let mut sub = thread.sub;
            loop {
                let inst = match self.next_inst() {
                    Some(inst) => inst,
                    None => return false,
                };
                match *inst {
                    Inst::Char(ch) => {
                        if self.next_char() != Some(ch) {
                            break;
                        }
                    }
                    Inst::Any => {
                        if self.next_char().is_none() {
                            break;
                        }
                    }
                    Inst::Match => {
                        (*sub).clone_into(sub_out);
                        return true;
                    }
                    Inst::Jmp(x) => self.pc = x,
                    Inst::Split(x, y) => {
                        if ready.len() >= MAX_THREADS {
                            panic!("backtrack overflow");
                        }
                        ready.push(Thread {
                            pc: y,
                            offset: self.offset,
                            sub: sub.clone(),
                        });
                        self.pc = x;
                    }
                    Inst::Save(n) => {
                        sub = sub.update(n, self.offset);
                    }
                }
            }
        }
        false
    }
}
