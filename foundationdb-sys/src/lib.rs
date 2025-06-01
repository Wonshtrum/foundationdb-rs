#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::unreadable_literal)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const FDB_API_VERSION: u32 = FDB_LATEST_API_VERSION;
