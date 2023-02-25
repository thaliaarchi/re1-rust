// Copyright 2023 Thalia Archibald. All Rights Reserved.
// Copyright 2007-2009 Russ Cox. All Rights Reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::env;
use std::process::exit;

use re1::{match_recursive, Regexp};

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
        Ok(re) => re,
        Err(err) => {
            eprintln!("parse: {}", err);
            exit(1);
        }
    };
    println!("{re}");
    let mut prog = re.compile();
    println!("{prog}");
    for s in args {
        println!("Matching {s}");
        prog.pc = 0;
        println!(
            "recursive {}",
            match_recursive(&mut prog, s.char_indices(), &mut [])
        );
    }
}
