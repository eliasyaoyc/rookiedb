#![allow(dead_code)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::await_holding_lock)]
#![feature(new_uninit)]
#![feature(slice_as_chunks)]
use std::fmt::Debug;

mod catalog;
mod database;
mod datatypes;
pub mod error;
pub(crate) mod query;
mod table;
mod utils;

pub struct Options {
    path: String,
    num_records_per_page: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            path: todo!(),
            num_records_per_page: todo!(),
        }
    }
}

impl Debug for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Options").finish()
    }
}
