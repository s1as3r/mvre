use std::io::{self, Write};
use std::path::Path;

pub(crate) fn prompt_user(msg: &str) -> bool {
    print!("{} [y/N]: ", msg);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().eq_ignore_ascii_case("y")
}

#[cfg(windows)]
pub(crate) fn is_hidden(p: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x00000002;
    p.metadata()
        .map(|m| m.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0)
        .unwrap_or(false)
}

#[cfg(unix)]
pub(crate) fn is_hidden(p: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt;

    p.file_name()
        .map(|name| name.as_bytes().first() == Some(&b'.'))
        .unwrap_or(false)
}
