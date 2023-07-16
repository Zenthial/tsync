use crate::channel::close_channel;
use crate::watcher::{SyncEvent, SyncFileType};

use ssh2::{Channel, Session};

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

fn create_file(sess: &mut Session, contents: String, path: PathBuf) {
    let bytes = contents.as_bytes();
    let mut file = sess
        .scp_send(&path, 0o644, bytes.len() as u64, None)
        .unwrap();

    file.write_all(bytes).unwrap();
    file.send_eof().unwrap();
    file.wait_eof().unwrap();
    file.close().unwrap();
    file.wait_close().unwrap();
}

fn create_folder(chan: &mut Channel, path: PathBuf) {
    chan.exec(&format!("mkdir -p {}", path.to_str().unwrap()))
        .unwrap();
}

fn remove_file(chan: &mut Channel, path: PathBuf) {
    chan.exec(&format!("rm -f {}", path.to_str().unwrap()))
        .unwrap();
}

fn remove_folder(chan: &mut Channel, path: PathBuf) {
    chan.exec(&format!("rm -rf {}", path.to_str().unwrap()))
        .unwrap();
}

fn modify(sess: &mut Session, chan: &mut Channel, contents: String, path: PathBuf) {
    // this will remove both files and folders
    remove_folder(chan, path.clone());
    create_file(sess, contents, path);
}

pub fn transform_path(path: PathBuf, src: &Path, dest: &Path) -> PathBuf {
    let path_str = path.to_str().unwrap().to_string();
    let dest_str = dest.to_str().unwrap();
    let src_str = src.to_str().unwrap();
    let replaced_path = path_str.replace(src_str, dest_str);
    PathBuf::from(replaced_path)
}

pub fn sync_missing_paths(
    sess: &mut Session,
    missing_paths: Vec<PathBuf>,
    src: &Path,
    dest: &Path,
) {
    for path in missing_paths {
        if path.is_dir() {
            let mut chan = sess.channel_session().unwrap();
            create_folder(&mut chan, transform_path(path, src, dest));
            close_channel(&mut chan);
        } else if path.is_file() {
            let contents = fs::read_to_string(&path).unwrap();
            create_file(sess, contents, transform_path(path, src, dest));
        }
    }
}

pub fn handle_event(sess: &mut Session, event: SyncEvent, src: PathBuf, dest: PathBuf) {
    let mut channel = sess.channel_session().unwrap();
    match event {
        // this code is almost identical to the next match statement
        // could probably learn to write macros, and abstract this better
        SyncEvent::Create(path, file_type) => match file_type {
            SyncFileType::File => {
                let contents = fs::read_to_string(&path).unwrap();
                create_file(sess, contents, transform_path(path, &src, &dest))
            }
            SyncFileType::Folder => create_folder(&mut channel, transform_path(path, &src, &dest)),
            SyncFileType::Any => {
                if path.is_file() {
                    let contents = fs::read_to_string(&path).unwrap();
                    create_file(sess, contents, transform_path(path, &src, &dest))
                } else if path.is_dir() {
                    create_folder(&mut channel, transform_path(path, &src, &dest))
                }
            }
        },
        SyncEvent::Remove(path, file_type) => match file_type {
            SyncFileType::File => remove_file(&mut channel, transform_path(path, &src, &dest)),
            SyncFileType::Folder => remove_folder(&mut channel, transform_path(path, &src, &dest)),
            SyncFileType::Any => {
                if path.is_file() {
                    remove_file(&mut channel, transform_path(path, &src, &dest))
                } else if path.is_dir() {
                    remove_folder(&mut channel, transform_path(path, &src, &dest))
                }
            }
        },
        SyncEvent::DataChange(path) | SyncEvent::Modify(path) => {
            if path.is_file() {
                let contents = fs::read_to_string(&path).unwrap();
                modify(
                    sess,
                    &mut channel,
                    contents,
                    transform_path(path, &src, &dest),
                )
            }
        }
    }

    close_channel(&mut channel);
}
