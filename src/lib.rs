#![allow(dead_code)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::await_holding_lock)]
#![feature(new_uninit)]
#![feature(maybe_uninit_slice)]
mod catalog;
mod common;
mod database;
pub mod error;
pub mod options;
pub mod query;
mod table;
mod utils;
