#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

pub mod api;
pub mod cli;
//pub mod tui;

pub mod board;
pub use crate::board::Board;

pub mod ai;
