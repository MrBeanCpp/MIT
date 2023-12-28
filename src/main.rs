mod cli;
mod commands;
mod models;
mod utils;

fn main() {
    color_backtrace::install(); // colorize backtrace
    cli::handle_command();
    models::Index::get_instance().save(); //兜底save
}
