use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::util::prompt_user;

pub(crate) fn compute_new_path(
    base_dir: &Path,
    path: &Path,
    src_re: &regex::Regex,
    dest_re_str: &str,
    files_only: bool,
    dirs_only: bool,
) -> Option<PathBuf> {
    // compute path string to match against by stripping the base directory.
    // this ensures the regex only applies to the path relative to the search directory.
    let match_path = path.strip_prefix(base_dir).unwrap_or(path);
    let path_str = match_path.to_string_lossy();

    if (path.is_dir() && files_only) || (path.is_file() && dirs_only) {
        return None;
    }

    if !src_re.is_match(&path_str) {
        return None;
    }

    let new_path_str = src_re.replace_all(&path_str, dest_re_str);
    if new_path_str == path_str {
        return None;
    }

    Some(base_dir.join(new_path_str.as_ref()))
}

pub(crate) fn do_rename(
    path: &Path,
    new_path: &Path,
    interactive: bool,
    force: bool,
    dry_run: bool,
) -> io::Result<()> {
    if dry_run {
        return Ok(());
    }

    let display_dest = new_path.display();
    if new_path.exists() {
        if interactive {
            if !prompt_user("Destination already exists. Overwrite?") {
                return Ok(());
            }
        } else if force {
            log::warn!("Overwriting existing file {}", display_dest);
        } else {
            log::warn!("Destination {} already exists. Skipping.", display_dest);
            return Ok(());
        }
    } else if interactive && !prompt_user("Perform this rename?") {
        return Ok(());
    }

    if let Some(parent) = new_path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
        && let Err(e) = fs::create_dir_all(parent)
    {
        return Err(e);
    }

    fs::rename(path, new_path)
}

pub(crate) fn get_files_recursive(dir: &Path, no_ignore_hidden: bool) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    assert!(dir.is_dir(), "must only be called with directories.");
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !no_ignore_hidden && file_name.starts_with('.') {
            continue;
        }

        files.push(path.clone());

        if path.is_dir() {
            files.extend(get_files_recursive(&path, no_ignore_hidden)?);
        }
    }
    Ok(files)
}
