use crate::channel::close_channel;
use crate::watcher::{SyncEvent, SyncFileType};

use ssh2::{Channel, Error, Session};

use std::io::Read;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

fn close_file(file: &mut Channel) -> Result<(), Error> {
    file.send_eof()?;
    file.wait_eof()?;
    file.close()?;
    file.wait_close()?;

    Ok(())
}

fn create_file(sess: &mut Session, contents: String, path: PathBuf) {
    let bytes = contents.as_bytes();
    let mut file = sess
        .scp_send(&path, 0o644, bytes.len() as u64, None)
        .unwrap();

    if let Err(e) = file.write_all(bytes) {
        panic!("failed to create file\n  message: {}", e);
    }
    if let Err(e) = close_file(&mut file) {
        panic!("failed to close file\n  message: {}", e);
    }
}

fn create_folder(chan: &mut Channel, path: PathBuf) {
    chan.exec(&format!("mkdir -p {}", path.to_str().unwrap()))
        .expect("failed to exec command");
}

fn remove_file(chan: &mut Channel, path: PathBuf) {
    chan.exec(&format!("rm -f {}; echo $?", path.to_str().unwrap()))
        .expect("failed to exec command");
}

fn remove_folder(chan: &mut Channel, path: PathBuf) {
    chan.exec(&format!("rm -rf {}", path.to_str().unwrap()))
        .expect("failed to exec command");
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
            let contents = fs::read_to_string(&path).expect("failed to read to string");
            create_file(sess, contents, transform_path(path, src, dest));
        }
    }
}

pub fn sync_old_paths(sess: &mut Session, old_paths: Vec<PathBuf>, src: &Path, dest: &Path) {
    for path in old_paths {
        let mut chan = sess.channel_session().unwrap();

        let path_clone = path.clone();
        let path_str = path_clone.to_str().unwrap();
        let contents = fs::read_to_string(&path).expect("failed to read to string");
        modify(sess, &mut chan, contents, transform_path(path, &src, &dest));
        println!("updated contents: {path_str}");

        close_channel(&mut chan);
    }
}

pub fn handle_event(sess: &mut Session, event: SyncEvent, src: PathBuf, dest: PathBuf) {
    let mut channel = sess.channel_session().expect("failed to create channel");
    match event {
        // this code is almost identical to the next match statement
        // could probably learn to write macros, and abstract this better
        SyncEvent::Create(path, file_type) => match file_type {
            SyncFileType::File => {
                let contents = fs::read_to_string(&path).expect("failed to read to string");
                create_file(sess, contents, transform_path(path, &src, &dest))
            }
            SyncFileType::Folder => create_folder(&mut channel, transform_path(path, &src, &dest)),
            SyncFileType::Any => {
                if path.is_file() {
                    let contents = fs::read_to_string(&path).expect("failed to read to string");
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
                // will remove both files and folders
                remove_folder(&mut channel, transform_path(path, &src, &dest))
            }
        },
        SyncEvent::DataChange(path) | SyncEvent::Modify(path) => {
            if path.is_file() {
                let contents = fs::read_to_string(&path).expect("failed to read to string");
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
