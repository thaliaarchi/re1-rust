// Copyright 2023 Thalia Archibald. All Rights Reserved.
// Copyright 2007-2009 Russ Cox. All Rights Reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::mem;
use std::rc::Rc;

use crate::{Inst, Sub, VM};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Thread {
    pc: usize,
    sub: Rc<Sub>,
}

impl Thread {
    fn new(pc: usize, sub: Rc<Sub>) -> Self {
        Thread { pc, sub }
    }
}

impl VM<'_, '_> {
    pub fn match_pikevm(&mut self, sub_out: &mut Sub) -> bool {
        let mut curr_threads = Vec::new();
        let mut next_threads = Vec::new();
        // visited replaces global gen (generation) in the original
        let mut visited = vec![false; self.insts.len()];

        sub_out.reset();
        let t = Thread::new(0, Rc::new(Sub::new(sub_out.len())));
        add_thread(&mut curr_threads, t, &mut visited, self.insts, 0);

        let mut matched = None;
        loop {
            let ch = self.next_char();
            if curr_threads.len() == 0 {
                break;
            }
            visited.fill(false);
            for t in curr_threads.drain(..) {
                let pc = t.pc;
                match self.insts[pc] {
                    Inst::Char(ch1) => {
                        if ch == Some(ch1) {
                            let t = Thread::new(pc + 1, t.sub);
                            add_thread(&mut next_threads, t, &mut visited, self.insts, self.offset);
                        }
                    }
                    Inst::Any => {
                        if ch.is_some() {
                            let t = Thread::new(pc + 1, t.sub);
                            add_thread(&mut next_threads, t, &mut visited, self.insts, self.offset);
                        }
                    }
                    Inst::Match => {
                        matched = Some(t.sub);
                        break;
                    }
                    // Jmp, Split, Save handled in add_thread, so that
                    // machine execution matches what a backtracker would do.
                    // This is discussed (but not shown as code) in
                    // Regular Expression Matching: the Virtual Machine Approach.
                    _ => {}
                }
            }
            mem::swap(&mut curr_threads, &mut next_threads);
            next_threads.clear();
            if ch.is_none() {
                break;
            }
        }
        if let Some(sub) = matched {
            sub_out.clone_from(&sub);
            true
        } else {
            false
        }
    }
}

fn add_thread(l: &mut Vec<Thread>, t: Thread, visited: &mut [bool], insts: &[Inst], offset: usize) {
    let pc = t.pc;
    if visited[pc] {
        return; // already on list
    }
    visited[pc] = true;

    match &insts[pc] {
        Inst::Jmp(x) => add_thread(l, Thread::new(*x, t.sub), visited, insts, offset),
        Inst::Split(x, y) => {
            add_thread(l, Thread::new(*x, t.sub.clone()), visited, insts, offset);
            add_thread(l, Thread::new(*y, t.sub), visited, insts, offset);
        }
        Inst::Save(n) => {
            let sub = t.sub.update(*n, offset);
            add_thread(l, Thread::new(pc + 1, sub), visited, insts, offset);
        }
        _ => l.push(t),
    }
}
