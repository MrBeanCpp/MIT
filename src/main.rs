#![allow(clippy::bool_assert_comparison)] // see tree&false directly
#![allow(clippy::bool_comparison)] // see tree&false directly
mod cli;
mod commands;
mod models;
mod utils;

fn main() {
    color_backtrace::install(); // colorize backtrace
    cli::handle_command();
}
