mod watcher;

use clap::Parser;
use rpassword::prompt_password;
use ssh2::{KeyboardInteractivePrompt, Session};

use std::io::prelude::*;
use std::net::TcpStream;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
struct App {
    src: PathBuf,
    dest: PathBuf,
    server: String,
    username: String,
    port: Option<i32>,
}

struct Prompt;

impl KeyboardInteractivePrompt for Prompt {
    fn prompt<'a>(
        &mut self,
        _username: &str,
        _instructions: &str,
        prompts: &[ssh2::Prompt<'a>],
    ) -> Vec<String> {
        prompts
            .iter()
            .map(|p| {
                let prompt_text = p.text.to_string();
                prompt_password(prompt_text).unwrap()
            })
            .collect()
    }
}

fn main() {
    let args = App::parse();
    watcher::watch(args.src);
    // let tcp = TcpStream::connect(format!(
    //     "{}:{}",
    //     args.server,
    //     args.port.map_or_else(|| 22, |p| p)
    // ));
    //
    // let username = &args.username;
    // match tcp {
    //     Ok(con) => {
    //         println!("connected to goblin");
    //         let mut sess = Session::new().unwrap();
    //         sess.set_tcp_stream(con);
    //         sess.handshake().unwrap();
    //
    //         sess.auth_methods(username).unwrap();
    //         let mut p = Prompt {};
    //         sess.userauth_keyboard_interactive(username, &mut p)
    //             .unwrap();
    //     }
    //     Err(e) => {
    //         panic!("failed to connect to {} {}", args.server, e);
    //     }
    // }
    // let mut sess = Session::new().unwrap();
    // sess.set_tcp_stream(tcp);
    // sess.userauth_agent("tschollenberger_mgr").unwrap();
    // sess.handshake().unwrap();
    // println!("here");
    //
    // // Write the file
    // let mut remote_file = sess.scp_send(Path::new("remote"), 0o644, 10, None).unwrap();
    // remote_file.write(b"1234567890").unwrap();
    // // Close the channel and wait for the whole content to be transferred
    // remote_file.send_eof().unwrap();
    // remote_file.wait_eof().unwrap();
    // remote_file.close().unwrap();
    // remote_file.wait_close().unwrap();
}
