mod auth;
mod channel;
mod checker;
mod syncer;
mod watcher;

use clap::Parser;
use ssh2::Session;

use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

#[derive(Parser, Debug)]
struct App {
    src: PathBuf,
    dest: PathBuf,
    server: String,
    username: String,
    port: Option<i32>,
}

fn main() {
    let args = App::parse();

    let tcp = TcpStream::connect(format!(
        "{}:{}",
        args.server,
        args.port.map_or_else(|| 22, |p| p)
    ));

    let username = &args.username;
    let mut sess = match tcp {
        Ok(con) => {
            println!("connected to {}", args.server);
            let mut sess = Session::new().expect("failed to establish ssh session");
            sess.set_tcp_stream(con);
            sess.handshake()
                .expect("failed to establish session handshake");

            if let Err(e) = auth::authenticate(&mut sess, username) {
                panic!("failed to auth {}", e);
            }

            sess
        }
        Err(e) => {
            panic!("failed to connect to {} {}", args.server, e);
        }
    };
    // 15 minutes
    sess.set_keepalive(false, 900);

    let (sender, receiver) = mpsc::channel();
    let src = args.src.clone();
    thread::spawn(move || {
        let (missing_paths, old_paths) = checker::check(&mut sess, &src, &args.dest);
        syncer::sync_missing_paths(&mut sess, missing_paths, &src, &args.dest);
        syncer::sync_old_paths(&mut sess, old_paths, &src, &args.dest);

        for receive in receiver {
            syncer::handle_event(&mut sess, receive, src.clone(), args.dest.clone());
            // println!("received: {:?}", receive);
        }
    });

    println!("listening for changes on {}", args.src.to_str().unwrap());
    watcher::watch(args.src, sender);
}
