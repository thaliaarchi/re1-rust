// Copyright 2023 Thalia Archibald. All Rights Reserved.
// Copyright 2007-2009 Russ Cox. All Rights Reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use crate::{Inst, Prog, Regexp};

impl Regexp {
    pub fn compile(&self) -> Prog {
        let mut b = ProgBuilder {
            insts: Vec::with_capacity(self.count_insts() + 1),
            nsub: 0,
        };
        b.push_regexp(self);
        b.insts.push(Inst::Match);
        Prog::new(b.insts, b.nsub)
    }

    // Counts the number of instructions needed to compile the regexp.
    fn count_insts(&self) -> usize {
        match self {
            Regexp::Alt(left, right) => 2 + left.count_insts() + right.count_insts(),
            Regexp::Cat(left, right) => left.count_insts() + right.count_insts(),
            Regexp::Lit(_) => 1,
            Regexp::Dot => 1,
            Regexp::Paren(_, inner) => 2 + inner.count_insts(),
            Regexp::Quest(_, inner) => 1 + inner.count_insts(),
            Regexp::Star(_, inner) => 2 + inner.count_insts(),
            Regexp::Plus(_, inner) => 1 + inner.count_insts(),
        }
    }
}

struct ProgBuilder {
    insts: Vec<Inst>,
    nsub: usize,
}

impl ProgBuilder {
    fn push_regexp(&mut self, r: &Regexp) -> usize {
        let pc = self.insts.len();
        match r {
            Regexp::Alt(left, right) => {
                let split = self.push_split_placeholder();
                let x = self.push_regexp(left);
                let jmp = self.push_split_placeholder();
                let y = self.push_regexp(right);
                self.insts[split] = Inst::Split(x, y);
                self.insts[jmp] = Inst::Jmp(self.insts.len());
            }
            Regexp::Cat(left, right) => {
                self.push_regexp(left);
                self.push_regexp(right);
            }
            Regexp::Lit(ch) => self.insts.push(Inst::Char(*ch)),
            Regexp::Dot => self.insts.push(Inst::Any),
            Regexp::Paren(n, inner) => {
                self.insts.push(Inst::Save(2 * n));
                self.push_regexp(inner);
                self.insts.push(Inst::Save(2 * n + 1));
                self.nsub = self.nsub.max(2 * (n + 1));
            }
            Regexp::Quest(greedy, inner) => {
                let split = self.push_split_placeholder();
                let x = self.push_regexp(inner);
                self.insts[split] = if *greedy {
                    Inst::Split(x, self.insts.len())
                } else {
                    Inst::Split(self.insts.len(), x)
                };
            }
            Regexp::Star(greedy, inner) => {
                let split = self.push_split_placeholder();
                let x = self.push_regexp(inner);
                self.insts.push(Inst::Jmp(split));
                self.insts[split] = if *greedy {
                    Inst::Split(x, self.insts.len())
                } else {
                    Inst::Split(self.insts.len(), x)
                }
            }
            Regexp::Plus(greedy, inner) => {
                let x = self.push_regexp(inner);
                self.insts.push(if *greedy {
                    Inst::Split(x, self.insts.len() + 1)
                } else {
                    Inst::Split(self.insts.len() + 1, x)
                });
            }
        }
        pc
    }

    fn push_split_placeholder(&mut self) -> usize {
        let pc = self.insts.len();
        self.insts.push(Inst::Split(usize::MAX, usize::MAX));
        pc
    }
}
