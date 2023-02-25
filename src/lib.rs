// Copyright 2023 Thalia Archibald. All Rights Reserved.
// Copyright 2007-2009 Russ Cox. All Rights Reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

#[macro_use]
extern crate lalrpop_util;

mod compile;
mod lex;
lalrpop_mod!(parse);
mod recursive;
mod regexp;

pub use regexp::*;
