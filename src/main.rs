use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::Parser;
use regex::RegexBuilder;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// source regex
    src: String,

    /// destination regex
    dest: String,

    /// paths to search for files
    #[arg(default_value = ".")]
    paths: Vec<String>,

    /// ignore case
    #[arg(short, long)]
    case_insensitive: bool,

    /// run interactively
    #[arg(short, long)]
    interactive: bool,

    /// print each move
    #[arg(short, long)]
    verbose: bool,

    /// dry run - dont make any moves
    #[arg(long)]
    dry_run: bool,

    /// include hidden files and directories
    #[arg(short = 'H', long)]
    hidden: bool,

    /// force overwrite if destination already exists
    #[arg(short, long)]
    force: bool,

    /// match only files
    #[arg(long, conflicts_with = "dirs_only")]
    files_only: bool,

    /// match only directories
    #[arg(long, conflicts_with = "files_only")]
    dirs_only: bool,
}

fn main() {
    let args = Args::parse();

    let re = match RegexBuilder::new(&format!("^{}$", &args.src))
        .case_insensitive(args.case_insensitive)
        .build()
    {
        Ok(re) => re,
        Err(e) => {
            eprintln!("Error compiling source regex: {}", e);
            std::process::exit(1);
        }
    };

    let mut all_files = Vec::new();
    for path_str in &args.paths {
        let base_dir = Path::new(path_str);
        if !base_dir.exists() {
            eprintln!(
                "Warning: Search path {} does not exist. Skipping.",
                path_str
            );
            continue;
        }

        match get_files_recursive(base_dir, args.hidden) {
            Ok(f) => {
                for path in f {
                    all_files.push((base_dir.to_path_buf(), path));
                }
            }
            Err(e) => {
                eprintln!("Error reading directory {}: {}", path_str, e);
                std::process::exit(1);
            }
        }
    }

    let mut renames = Vec::new();
    for (base_dir, path) in &all_files {
        if let Some(new_path) = compute_new_path(base_dir, path, &re, &args) {
            renames.push((path.clone(), new_path));
        }
    }

    for (src, dest) in renames {
        do_rename(&src, &dest, &args);
    }
}

fn compute_new_path(
    base_dir: &Path,
    path: &Path,
    re: &regex::Regex,
    args: &Args,
) -> Option<PathBuf> {
    // compute path string to match against by stripping the base directory.
    // this ensures the regex only applies to the path relative to the search directory.
    let match_path = path.strip_prefix(base_dir).unwrap_or(path);
    let path_str = match_path.to_string_lossy();

    if (path.is_dir() && args.files_only) || (path.is_file() && args.dirs_only) {
        return None;
    }

    if !re.is_match(&path_str) {
        return None;
    }

    let new_path_str = re.replace_all(&path_str, args.dest.as_str());
    if new_path_str == path_str {
        return None;
    }

    Some(base_dir.join(new_path_str.as_ref()))
}

fn do_rename(path: &Path, new_path: &Path, args: &Args) {
    let display_src = path.display();
    let display_dest = new_path.display();

    if args.verbose || args.dry_run || args.interactive {
        println!("{} -> {}", display_src, display_dest);
    }

    if args.dry_run {
        return;
    }

    if new_path.exists() {
        if args.interactive {
            let msg = format!("Destination {} already exists. Overwrite?", display_dest);
            if !prompt_user(&msg) {
                return;
            }
        } else if args.force {
            if args.verbose {
                println!("  Warning: overwriting existing file {}", display_dest);
            }
        } else {
            eprintln!(
                "  Warning: Destination {} already exists. Skipping. Use --force to overwrite.",
                display_dest
            );
            return;
        }
    } else if args.interactive && !prompt_user("Perform this rename?") {
        return;
    }

    if let Some(parent) = new_path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
        && let Err(e) = fs::create_dir_all(parent)
    {
        eprintln!("  Error creating directory {}: {}", parent.display(), e);
        return;
    }

    if let Err(e) = fs::rename(path, new_path) {
        eprintln!(
            "  Error renaming {} to {}: {}",
            path.display(),
            new_path.display(),
            e
        );
    }
}

fn get_files_recursive(dir: &Path, no_ignore_hidden: bool) -> io::Result<Vec<PathBuf>> {
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

fn prompt_user(msg: &str) -> bool {
    print!("{} [y/N]: ", msg);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().eq_ignore_ascii_case("y")
}
