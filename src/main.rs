mod cli;
fn main() {
    color_backtrace::install(); // colorize backtrace
    cli::handle_command();
}
