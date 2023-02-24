// Copyright 2023 Thalia Archibald. All Rights Reserved.
// Copyright 2007-2009 Russ Cox. All Rights Reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::env;
use std::process::exit;

use re1::Regexp;

fn main() {
    let mut args = env::args();
    let re = match args.nth(1) {
        Some(re) => re,
        None => {
            eprintln!("usage: re1 <regexp> <string>...");
            exit(2);
        }
    };
    match Regexp::parse(&re) {
        Ok(re) => println!("{}", re),
        Err(err) => {
            eprintln!("parse: {}", err);
            exit(1);
        }
    }
}
