#![allow(dead_code)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::await_holding_lock)]
#![feature(new_uninit)]
use std::fmt::Debug;

mod catalog;
mod common;
mod database;
pub mod error;
pub mod query;
mod table;
mod utils;

pub struct Options {
    path: String,
    num_records_per_page: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            path: "".to_owned(),
            num_records_per_page: 8,
        }
    }
}

impl Debug for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Options").finish()
    }
}
