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
        self.add_thread(&mut curr_threads, Thread::new(0), &mut visited);

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
            for thread in &curr_threads {
                let pc = thread.pc;
                match self.insts[pc] {
                    Inst::Char(ch1) => {
                        if ch == Some(ch1) {
                            self.add_thread(&mut next_threads, Thread::new(pc + 1), &mut visited);
                        }
                    }
                    Inst::Any => {
                        if ch.is_some() {
                            self.add_thread(&mut next_threads, Thread::new(pc + 1), &mut visited);
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

    fn add_thread(&mut self, threads: &mut Vec<Thread>, thread: Thread, visited: &mut [bool]) {
        let pc = thread.pc;
        if visited[pc] {
            return; // already on list
        }
        visited[pc] = true;
        threads.push(thread);

        match &self.insts[pc] {
            Inst::Jmp(x) => self.add_thread(threads, Thread::new(*x), visited),
            Inst::Split(x, y) => {
                self.add_thread(threads, Thread::new(*x), visited);
                self.add_thread(threads, Thread::new(*y), visited);
            }
            Inst::Save(_) => self.add_thread(threads, Thread::new(pc + 1), visited),
            _ => {}
        }
    }
}
