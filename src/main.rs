// Copyright 2023 Thalia Archibald. All Rights Reserved.
// Copyright 2007-2009 Russ Cox. All Rights Reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::env;
use std::process::exit;

use re1::{Prog, Regexp, Sub, VM};

fn main() {
    let mut args = env::args();
    let pattern = match args.nth(1) {
        Some(pattern) => pattern,
        None => {
            eprintln!("usage: re1 <regexp> <string>...");
            exit(2);
        }
    };
    let re = match Regexp::parse(&pattern) {
        Ok(re) => re.unanchored(),
        Err(err) => {
            eprintln!("parse: {}", err);
            exit(1);
        }
    };
    println!("{re}\n");
    let prog = re.compile();
    print!("{prog}");
    let mut sub = Sub::new(prog.nsub());
    for (i, s) in args.enumerate() {
        println!("\n#{i} {s}");
        regexp_match("recursive", VM::match_recursive, &prog, &s, &mut sub);
        regexp_match(
            "recursiveloop",
            VM::match_recursive_loop,
            &prog,
            &s,
            &mut sub,
        );
        regexp_match("backtrack", VM::match_backtrack, &prog, &s, &mut sub);
        regexp_match("thompson", VM::match_thompsonvm, &prog, &s, &mut sub);
        regexp_match("pike", VM::match_pikevm, &prog, &s, &mut sub);
    }
}

fn regexp_match<'i, 's, F>(label: &str, matches: F, prog: &'i Prog, s: &'s str, sub: &mut Sub)
where
    F: FnOnce(&mut VM<'i, 's>, &mut Sub) -> bool,
{
    let mut vm = VM::new(&prog, &s, false);
    sub.reset();
    print!("{label} ");
    if matches(&mut vm, sub) {
        println!("match {sub}");
    } else {
        println!("-no match-");
    }
}
