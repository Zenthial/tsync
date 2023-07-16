use crate::watcher::{SyncEvent, SyncFileType};
use ssh2::{Channel, Session};

use std::{fs, io::Write, path::PathBuf};

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

pub fn transform_path(path: PathBuf, src: PathBuf, dest: PathBuf) -> PathBuf {
    let path_str = path.to_str().unwrap().to_string();
    let dest_str = dest.to_str().unwrap();
    let src_str = src.to_str().unwrap();
    let replaced_path = path_str.replace(src_str, dest_str);
    PathBuf::from(replaced_path)
}

pub fn handle_event(
    sess: &mut Session,
    channel: &mut Channel,
    event: SyncEvent,
    src: PathBuf,
    dest: PathBuf,
) {
    match event {
        // this code is almost identical to the next match statement
        // could probably learn to write a macro that abstracts this better
        SyncEvent::Create(path, file_type) => match file_type {
            SyncFileType::File => {
                let contents = fs::read_to_string(&path).unwrap();
                create_file(sess, contents, transform_path(path, src, dest))
            }
            SyncFileType::Folder => create_folder(channel, transform_path(path, src, dest)),
            SyncFileType::Any => {
                if path.is_file() {
                    let contents = fs::read_to_string(&path).unwrap();
                    create_file(sess, contents, transform_path(path, src, dest))
                } else if path.is_dir() {
                    create_folder(channel, transform_path(path, src, dest))
                }
            }
        },
        SyncEvent::Remove(path, file_type) => match file_type {
            SyncFileType::File => remove_file(channel, transform_path(path, src, dest)),
            SyncFileType::Folder => remove_folder(channel, transform_path(path, src, dest)),
            SyncFileType::Any => {
                if path.is_file() {
                    remove_file(channel, transform_path(path, src, dest))
                } else if path.is_dir() {
                    remove_folder(channel, transform_path(path, src, dest))
                }
            }
        },
        SyncEvent::DataChange(path) | SyncEvent::Modify(path) => {
            if path.is_file() {
                let contents = fs::read_to_string(&path).unwrap();
                modify(sess, channel, contents, transform_path(path, src, dest))
            }
        }
    }
}
