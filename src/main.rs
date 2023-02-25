// Copyright 2023 Thalia Archibald. All Rights Reserved.
// Copyright 2007-2009 Russ Cox. All Rights Reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::env;
use std::process::exit;

use re1::{Regexp, VM};

fn main() {
    let mut args = env::args();
    let pattern = match args.nth(1) {
        Some(pattern) => pattern,
        None => {
            eprintln!("usage: re1 <regexp> <string>...");
            exit(2);
        }
    };
    let re = match Regexp::parse_wrapped(&pattern) {
        Ok(re) => re,
        Err(err) => {
            eprintln!("parse: {}", err);
            exit(1);
        }
    };
    println!("{re}\n");
    let prog = re.compile();
    println!("{prog}");
    for s in args {
        println!("Matching {s}");
        let mut vm = VM::new(&prog, &s);
        println!("recursive {}", vm.match_recursive(&mut []));
        vm.reset();
        println!("recursiveloop {}", vm.match_recursive_loop(&mut []));
    }
}
