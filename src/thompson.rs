// Copyright 2023 Thalia Archibald. All Rights Reserved.
// Copyright 2007-2009 Russ Cox. All Rights Reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::mem;

use crate::{Inst, Sub, VM};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Thread {
    pc: usize,
}

impl Thread {
    fn new(pc: usize) -> Self {
        Thread { pc }
    }
}

impl VM<'_, '_> {
    pub fn match_thompsonvm(&mut self, sub_out: &mut Sub) -> bool {
        let mut curr_threads = Vec::new();
        let mut next_threads = Vec::new();
        // visited replaces global gen (generation) in the original
        let mut visited = vec![false; self.insts.len()];
        add_thread(&mut curr_threads, Thread::new(0), &mut visited, self.insts);

        sub_out.reset();
        if sub_out.len() >= 1 {
            sub_out.set(0, 0);
        }

        let mut matched = false;
        loop {
            let offset = self.offset;
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
                            let t = Thread::new(pc + 1);
                            add_thread(&mut next_threads, t, &mut visited, self.insts);
                        }
                    }
                    Inst::Any => {
                        if ch.is_some() {
                            let t = Thread::new(pc + 1);
                            add_thread(&mut next_threads, t, &mut visited, self.insts);
                        }
                    }
                    Inst::Match => {
                        if sub_out.len() >= 2 {
                            sub_out.set(1, offset);
                        }
                        matched = true;
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
        matched
    }
}

fn add_thread(l: &mut Vec<Thread>, t: Thread, visited: &mut [bool], insts: &[Inst]) {
    let pc = t.pc;
    if visited[pc] {
        return; // already on list
    }
    visited[pc] = true;
    l.push(t);

    match &insts[pc] {
        Inst::Jmp(x) => add_thread(l, Thread::new(*x), visited, insts),
        Inst::Split(x, y) => {
            add_thread(l, Thread::new(*x), visited, insts);
            add_thread(l, Thread::new(*y), visited, insts);
        }
        Inst::Save(_) => add_thread(l, Thread::new(pc + 1), visited, insts),
        _ => {}
    }
}
