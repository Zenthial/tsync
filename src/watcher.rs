use notify::{event, Config, Event, EventKind, PollWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

fn create(path: &PathBuf, create_kind: event::CreateKind) {}

fn handle_event(event: Event) {
    if event.need_rescan() {
        return;
    }

    match event.kind {
        EventKind::Create(c_kind) => {
            create(&event.paths[0], c_kind);
        }
        EventKind::Modify(m_kind) => {}
        EventKind::Remove(r_kind) => {}
        _ => {}
    }
}

pub fn watch(path: PathBuf) {
    let config = Config::default()
        .with_compare_contents(true)
        .with_poll_interval(Duration::from_secs(1));

    let (sender, receiver) = mpsc::channel();

    let mut watcher = PollWatcher::new(sender, config).unwrap();
    watcher.watch(&path, RecursiveMode::Recursive).unwrap();

    // never returns
    for result in receiver {
        match result {
            Ok(event) => println!("{:?}", event),
            Err(e) => println!("watch error {:?}", e),
        }
    }
}
