use rusty_v8 as v8;
// Copyright 2018-2020 the Deno authors. All rights reserved. MIT license.

pub const DENO: &str = env!("CARGO_PKG_VERSION");
//pub const TYPESCRIPT: &str = crate::js::TS_VERSION;

pub fn v8() -> &'static str {
    v8_version()
}

pub fn v8_version() -> &'static str {
    v8::V8::get_version()
}
