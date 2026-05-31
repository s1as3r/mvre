# `mvre` - `mv` using `re`gex

Move files using regexes.

I use this for batch renaming files.

The supplied `src` regex is anchored with `^` and `$` so that only paths that
completely match are moved.

> [!CAUTION]
> I have not extensively tested this yet. This was hastily written and might
> cause some unwanted moves. Always run with `--dry-run` first or
> `--interactive` to make sure only moves you anticipate are happening.

## Usage & Examples
```
mvre [OPTIONS] <SRC> <DEST> [PATHS]...

Arguments:
  <SRC>       source regex
  <DEST>      destination regex
  [PATHS]...  paths to search for files [default: .]

Options:
  -c, --case-insensitive  ignore case
  -i, --interactive       run interactively
  -v, --verbose           print each move
      --dry-run           dry run - dont make any moves
  -H, --hidden            include hidden files and directories
  -f, --force             force overwrite if destination already exists
      --files-only        match only files
      --dirs-only         match only directories
  -h, --help              Print help
  -V, --version           Print version
```

- Change all `.jpeg` extensions to `.jpg`:
  ```bash
  mvre "(.*)\.jpeg" "$1.jpg"
  ```

- Rename files across multiple, separate directories at once:
  ```bash
  mvre "(.*)logo-old\.png" "${1}logo-new.png" public/ src/assets/ dist/
  ```

> [!TIP]
> Why use `--dirs-only` / `--files-only`?
> If you rename a parent directory, the files inside it "move" with it. If your
> regex matches *both* the directory and the files inside, `mvre` will show a warning
> when it tries to rename a file whose parent directory was already renamed.
