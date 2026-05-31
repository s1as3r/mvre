use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_basic_rename() -> Result<()> {
    let dir = tempdir()?;
    let movies_dir = dir.path().join("movies");
    fs::create_dir(&movies_dir)?;
    fs::write(movies_dir.join("23nametogether_2.mp4"), "content")?;

    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg("(.*?)/.*?_(\\d+)\\.(.*?)")
        .arg("${1}_${2}.${3}")
        .assert()
        .success();

    assert!(dir.path().join("movies_2.mp4").exists());
    assert!(!dir.path().join("movies/23nametogether_2.mp4").exists());
    Ok(())
}

#[test]
fn test_ignore_hidden_files_by_default() -> Result<()> {
    let dir = tempdir()?;
    let hidden_dir = dir.path().join(".hidden");
    fs::create_dir(&hidden_dir)?;
    fs::write(hidden_dir.join("file_1.txt"), "content")?;

    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg(r".*/file_(\d+)\.txt")
        .arg("file_$1.txt")
        .assert()
        .success();

    assert!(!dir.path().join("file_1.txt").exists());
    assert!(dir.path().join(".hidden/file_1.txt").exists());
    Ok(())
}

#[test]
fn test_include_hidden_files() -> Result<()> {
    let dir = tempdir()?;
    let hidden_dir = dir.path().join(".hidden");
    fs::create_dir(&hidden_dir)?;
    fs::write(hidden_dir.join("file_1.txt"), "content")?;

    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg("-H")
        .arg(r".*/file_(\d+)\.txt")
        .arg("file_$1.txt")
        .assert()
        .success();

    assert!(dir.path().join("file_1.txt").exists());
    assert!(!dir.path().join(".hidden/file_1.txt").exists());
    Ok(())
}

#[test]
fn test_collision_skip() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("file_1.txt"), "source")?;
    fs::write(dir.path().join("target_1.txt"), "existing")?;

    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg(r"file_(\d+)\.txt")
        .arg("target_$1.txt")
        .assert()
        .success()
        .stderr(predicate::str::contains("already exists"));

    assert!(dir.path().join("file_1.txt").exists());
    assert_eq!(
        fs::read_to_string(dir.path().join("target_1.txt"))?,
        "existing"
    );
    Ok(())
}

#[test]
fn test_collision_overwrite() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("file_1.txt"), "source")?;
    fs::write(dir.path().join("target_1.txt"), "existing")?;

    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg("-f")
        .arg(r"file_(\d+)\.txt")
        .arg("target_$1.txt")
        .assert()
        .success();

    assert!(!dir.path().join("file_1.txt").exists());
    assert_eq!(
        fs::read_to_string(dir.path().join("target_1.txt"))?,
        "source"
    );
    Ok(())
}

#[test]
fn test_interactive_collision_overwrite_yes() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("file_1.txt"), "source")?;
    fs::write(dir.path().join("target_1.txt"), "existing")?;

    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg("-i")
        .arg(r"file_(\d+)\.txt")
        .arg("target_$1.txt")
        .write_stdin("y\n")
        .assert()
        .success();

    assert!(!dir.path().join("file_1.txt").exists());
    assert_eq!(
        fs::read_to_string(dir.path().join("target_1.txt"))?,
        "source"
    );
    Ok(())
}

#[test]
fn test_interactive_collision_overwrite_no() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("file_1.txt"), "source")?;
    fs::write(dir.path().join("target_1.txt"), "existing")?;

    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg("-i")
        .arg(r"file_(\d+)\.txt")
        .arg("target_$1.txt")
        .write_stdin("n\n")
        .assert()
        .success();

    assert!(dir.path().join("file_1.txt").exists());
    assert_eq!(
        fs::read_to_string(dir.path().join("target_1.txt"))?,
        "existing"
    );
    Ok(())
}

#[test]
fn test_dry_run() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("file_1.txt"), "source")?;

    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg("--dry-run")
        .arg(r"file_(\d+)\.txt")
        .arg("target_$1.txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("./file_1.txt -> ./target_1.txt"));

    assert!(dir.path().join("file_1.txt").exists());
    assert!(!dir.path().join("target_1.txt").exists());
    Ok(())
}

#[test]
fn test_create_parent_directories() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("file_1.txt"), "source")?;

    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg(r"file_(\d+)\.txt")
        .arg("new_dir/sub_dir/target_$1.txt")
        .assert()
        .success();

    assert!(dir.path().join("new_dir/sub_dir/target_1.txt").exists());
    assert!(!dir.path().join("file_1.txt").exists());
    Ok(())
}

#[test]
fn test_case_insensitive() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("FiLe_1.TXT"), "source")?;

    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg("-c")
        .arg(r"file_(\d+)\.txt")
        .arg("target_$1.txt")
        .assert()
        .success();

    assert!(dir.path().join("target_1.txt").exists());
    assert!(!dir.path().join("FiLe_1.TXT").exists());
    Ok(())
}

#[test]
fn test_explicit_path_argument() -> Result<()> {
    let dir = tempdir()?;
    let external_dir = dir.path().join("external_dir");
    fs::create_dir(&external_dir)?;
    fs::write(external_dir.join("file_1.txt"), "source")?;

    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg(r"file_(\d+)\.txt")
        .arg("target_$1.txt")
        .arg("external_dir")
        .assert()
        .success();

    assert!(dir.path().join("external_dir/target_1.txt").exists());
    assert!(!dir.path().join("external_dir/file_1.txt").exists());
    Ok(())
}

#[test]
fn test_explicit_absolute_path() -> Result<()> {
    let dir = tempdir()?;
    let external_dir = dir.path().join("external_dir");
    fs::create_dir(&external_dir)?;
    let file_path = external_dir.join("file_1.txt");
    fs::write(&file_path, "source")?;

    let mut cmd = Command::cargo_bin("mvre")?;

    let regex_str = r"(.*)file_(\d+)\.txt".to_string();

    cmd.current_dir(dir.path())
        .arg(&regex_str)
        .arg("${1}target_$2.txt")
        .arg(external_dir.to_str().unwrap())
        .assert()
        .success();

    assert!(dir.path().join("external_dir/target_1.txt").exists());
    assert!(!dir.path().join("external_dir/file_1.txt").exists());
    Ok(())
}

#[test]
fn test_files_only() -> Result<()> {
    let dir = tempdir()?;
    let my_dir = dir.path().join("match_1");
    fs::create_dir(&my_dir)?;
    fs::write(dir.path().join("match_2"), "source")?;

    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg("--files-only")
        .arg(r"match_(\d+)")
        .arg("target_$1")
        .assert()
        .success();

    assert!(dir.path().join("match_1").exists());
    assert!(!dir.path().join("target_1").exists());

    assert!(!dir.path().join("match_2").exists());
    assert!(dir.path().join("target_2").exists());

    Ok(())
}

#[test]
fn test_dirs_only() -> Result<()> {
    let dir = tempdir()?;
    let my_dir = dir.path().join("match_1");
    fs::create_dir(&my_dir)?;
    fs::write(dir.path().join("match_2"), "source")?;

    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg("--dirs-only")
        .arg(r"match_(\d+)")
        .arg("target_$1")
        .assert()
        .success();

    assert!(!dir.path().join("match_1").exists());
    assert!(dir.path().join("target_1").exists());

    assert!(dir.path().join("match_2").exists());
    assert!(!dir.path().join("target_2").exists());

    Ok(())
}

#[test]
fn test_verbose() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("file_1.txt"), "source")?;

    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg("--verbose")
        .arg(r"file_(\d+)\.txt")
        .arg("target_$1.txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("./file_1.txt -> ./target_1.txt"));

    assert!(dir.path().join("target_1.txt").exists());
    Ok(())
}

#[test]
fn test_interactive_rename_yes() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("file_1.txt"), "source")?;

    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg("-i")
        .arg(r"file_(\d+)\.txt")
        .arg("target_$1.txt")
        .write_stdin("y\n")
        .assert()
        .success();

    assert!(!dir.path().join("file_1.txt").exists());
    assert!(dir.path().join("target_1.txt").exists());
    Ok(())
}

#[test]
fn test_interactive_rename_no() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join("file_1.txt"), "source")?;

    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg("-i")
        .arg(r"file_(\d+)\.txt")
        .arg("target_$1.txt")
        .write_stdin("n\n")
        .assert()
        .success();

    assert!(dir.path().join("file_1.txt").exists());
    assert!(!dir.path().join("target_1.txt").exists());
    Ok(())
}

#[test]
fn test_files_only_and_dirs_only_conflict() -> Result<()> {
    let dir = tempdir()?;
    let mut cmd = Command::cargo_bin("mvre")?;
    cmd.current_dir(dir.path())
        .arg("--files-only")
        .arg("--dirs-only")
        .arg("src")
        .arg("dest")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));

    Ok(())
}
