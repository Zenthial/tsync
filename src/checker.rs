use crate::channel::close_channel;
use crate::syncer::transform_path;

use ssh2::{Channel, Session};
use walkdir::WalkDir;

use std::fs;
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
    let code_char = &ls_err_code[0..1];
    let code = code_char.parse::<i32>().unwrap();

    code == 0
}

fn check_matches(sess: &mut Session, src: &Path, dest: &Path) -> bool {
    let src_contents = fs::read_to_string(src).unwrap();
    let mut dest_contents = String::new();

    let (mut dest_file, _stat) = sess.scp_recv(dest).unwrap();
    dest_file.read_to_string(&mut dest_contents).unwrap();

    src_contents == dest_contents
}

fn walk_dir(sess: &mut Session, dir: &Path, dest: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut missing = Vec::new();
    let mut mismatch = Vec::new();
    for entry in WalkDir::new(dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();
        let transformed_path = &transform_path(entry_path.to_path_buf(), dir, dest);

        let mut chan = sess.channel_session().unwrap();
        if !exists(&mut chan, transformed_path) {
            missing.push(entry_path.to_path_buf());
        } else if entry_path.is_file() && !check_matches(sess, entry_path, transformed_path) {
            mismatch.push(entry_path.to_path_buf());
        }

        close_channel(&mut chan);
    }

    (missing, mismatch)
}

pub fn check(sess: &mut Session, src: &Path, dest: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    if src.is_dir() {
        walk_dir(sess, src, dest)
    } else if src.is_file() {
        unimplemented!()
    } else {
        panic!("source is not a file or a directory!");
    }
}
