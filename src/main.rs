use std::collections::HashMap;
type Index = HashMap<String, bool>;
mod cli;
fn main() {
    cli::handle_command();
}
