use notify::{event, Config, Event, EventKind, PollWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

#[derive(Debug)]
pub enum SyncFileType {
    Any,
    File,
    Folder,
}

#[derive(Debug)]
pub enum SyncEvent {
    Create(PathBuf, SyncFileType),
    Remove(PathBuf, SyncFileType),
    Modify(PathBuf),
    DataChange(PathBuf),
}

fn create(path: &Path, create_kind: event::CreateKind) -> Option<SyncEvent> {
    match create_kind {
        event::CreateKind::File => Some(SyncEvent::Create(path.to_path_buf(), SyncFileType::File)),
        event::CreateKind::Folder => {
            Some(SyncEvent::Create(path.to_path_buf(), SyncFileType::Folder))
        }
        event::CreateKind::Any => Some(SyncEvent::Create(path.to_path_buf(), SyncFileType::Any)),
        _ => None,
    }
}

fn modify(paths: &[PathBuf], modify_kind: event::ModifyKind) -> Option<SyncEvent> {
    match modify_kind {
        event::ModifyKind::Data(_change) => Some(SyncEvent::DataChange(paths[0].clone())),
        event::ModifyKind::Name(rename_mode) => {
            println!("have never seen a rename: {:?}", rename_mode);
            None
        }
        event::ModifyKind::Any => Some(SyncEvent::Modify(paths[0].clone())),
        event::ModifyKind::Metadata(_) => Some(SyncEvent::Modify(paths[0].clone())),
        _ => None,
    }
}

fn remove(path: &Path, remove_kind: event::RemoveKind) -> Option<SyncEvent> {
    match remove_kind {
        event::RemoveKind::File => Some(SyncEvent::Remove(path.to_path_buf(), SyncFileType::File)),
        event::RemoveKind::Folder => {
            Some(SyncEvent::Remove(path.to_path_buf(), SyncFileType::Folder))
        }
        event::RemoveKind::Any => Some(SyncEvent::Remove(path.to_path_buf(), SyncFileType::Any)),
        _ => None,
    }
}

fn handle_event(event: Event) -> Option<SyncEvent> {
    if event.need_rescan() {
        return None;
    }

    // println!("{:?} {:?}", event.kind, event.paths);
    // A note, some operating systems will put renames as create and deletes
    match event.kind {
        EventKind::Create(c_kind) => create(&event.paths[0], c_kind),
        EventKind::Modify(m_kind) => modify(&event.paths, m_kind),
        EventKind::Remove(r_kind) => remove(&event.paths[0], r_kind),
        _ => None,
    }
}

pub fn watch(path: PathBuf, ssh_sender: Sender<SyncEvent>) {
    let config = Config::default()
        .with_compare_contents(true)
        .with_poll_interval(Duration::from_secs(1));

    let (sender, receiver) = mpsc::channel();

    let mut watcher = PollWatcher::new(sender, config).unwrap();
    watcher.watch(&path, RecursiveMode::Recursive).unwrap();

    // never returns
    for result in receiver {
        match result {
            Ok(event) => {
                if let Some(e) = handle_event(event) {
                    ssh_sender.send(e).unwrap()
                }
            }
            Err(_e) => {} //println!("watch error {:?}", e),
        }
    }
}
