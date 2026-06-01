use std::path::Path;

use clap::Parser;
use regex::RegexBuilder;

mod fs;
mod util;

fn main() {
    let args = Args::parse();

    if let Err(e) = simple_logger::init_with_level(if args.debug {
        log::Level::Debug
    } else if args.quiet {
        log::Level::Warn
    } else {
        log::Level::Info
    }) {
        eprintln!("Error initialising logger: {}", e);
    }

    let re = match RegexBuilder::new(&format!("^{}$", &args.src))
        .case_insensitive(args.case_insensitive)
        .build()
    {
        Ok(re) => re,
        Err(e) => {
            log::error!("Compiling source regex: {}", e);
            std::process::exit(1);
        }
    };

    let mut all_paths = Vec::new();
    for path_str in &args.paths {
        let base_dir = Path::new(path_str);
        if !base_dir.exists() {
            log::warn!("Search path {} does not exist. Skipping.", path_str);
            continue;
        }

        match fs::get_files_recursive(base_dir, args.hidden) {
            Ok(f) => {
                for path in f {
                    all_paths.push((base_dir, path));
                }
            }
            Err(e) => {
                log::error!("Reading directory {}: {}", path_str, e);
                std::process::exit(1);
            }
        }
    }

    let mut renames = Vec::new();
    for (base_dir, path) in &all_paths {
        if let Some(new_path) = fs::compute_new_path(
            base_dir,
            path,
            &re,
            &args.dest,
            args.files_only,
            args.dirs_only,
        ) {
            renames.push((path.clone(), new_path));
        }
    }

    for (src, dest) in renames {
        if !args.quiet {
            println!("{} -> {}", src.display(), dest.display());
        }
        if let Err(e) = fs::do_rename(&src, &dest, args.interactive, args.force, args.dry_run) {
            log::error!("Renaming {} to {}: {}", src.display(), dest.display(), e);
        }
    }
}

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
    #[arg(short, long, conflicts_with = "quiet")]
    interactive: bool,

    /// enable debug output
    #[arg(short, long, conflicts_with = "quiet")]
    debug: bool,

    /// suppress info output
    #[arg(short, long, conflicts_with = "interactive", conflicts_with = "debug")]
    quiet: bool,

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
