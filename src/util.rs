use std::io::{self, Write};

pub(crate) fn prompt_user(msg: &str) -> bool {
    print!("{} [y/N]: ", msg);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().eq_ignore_ascii_case("y")
}
