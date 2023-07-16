use crate::syncer::transform_path;

use ssh2::{Channel, Session};
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
    let code_char = &ls_err_code[0..1];
    let code = code_char.parse::<i32>().unwrap();
    println!("{} {}", code_char, code);
    return code == 0;
}

fn walk_dir(sess: &mut Session, dir: &Path, dest: &Path) -> Vec<PathBuf> {
    let mut missing = Vec::new();
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
        }
        chan.send_eof().unwrap();
        chan.wait_eof().unwrap();
        chan.close().unwrap();
        chan.wait_close().unwrap();
        chan.wait_close().unwrap();
    }

    missing
}

pub fn check(sess: &mut Session, src: &Path, dest: &Path) -> Vec<PathBuf> {
    if src.is_dir() {
        walk_dir(sess, src, dest)
    } else if src.is_file() {
        unimplemented!()
    } else {
        panic!("source is not a file or a directory!");
    }
}
