use mit::models::Index;

mod cli;
fn main() {
    color_backtrace::install(); // colorize backtrace
    cli::handle_command();
    Index::get_instance().save(); //兜底save
}
