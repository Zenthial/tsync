use crate::syncer::transform_path;

use ssh2::Channel;
use walkdir::WalkDir;

use std::io::Read;
use std::path::{Path, PathBuf};

fn exists(chan: &mut Channel, dest: &Path) -> bool {
    chan.exec(&format!(
        "ls {} > /dev/null;echo $?",
        dest.to_str().unwrap()
    ))
    .unwrap();

    let mut ls_err_code = String::new();
    chan.read_to_string(&mut ls_err_code).unwrap();
    let code = ls_err_code.parse::<i32>().unwrap();
    return code == 2;
}

fn walk_dir(chan: &mut Channel, dir: &Path, dest: &Path) -> Vec<PathBuf> {
    let mut missing = Vec::new();
    for entry in WalkDir::new(dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();
        let transformed_path = &transform_path(
            entry_path.to_path_buf(),
            dir.to_path_buf(),
            dest.to_path_buf(),
        );

        if !exists(chan, transformed_path) {
            missing.push(entry_path.to_path_buf());
        }
    }

    missing
}

fn check(chan: &mut Channel, src: &Path, dest: &Path) {
    if src.is_dir() {
        walk_dir(chan, src, dest);
    } else if src.is_file() {
    } else {
        panic!("source is not a file or a directory!");
    }
}
