// use promptly::prompt;
// use requestty::{prompt_one, Question};
use rpassword::prompt_password;
use ssh2::{KeyboardInteractivePrompt, Session};

use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;

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
    let tcp = TcpStream::connect("goblin.ecru.cert.org:22");
    match tcp {
        Ok(con) => {
            println!("connected to goblin");
            let mut sess = Session::new().unwrap();
            sess.set_tcp_stream(con);
            println!("{:?}", sess.handshake().unwrap());
            println!("{:?}", sess.auth_methods("tschollenberger_mgr"));
            let mut p = Prompt {};
            println!(
                "{:?}",
                sess.userauth_keyboard_interactive("tschollenberger_mgr", &mut p)
            );
            println!("here");
        }
        Err(e) => {
            panic!("failed to connect to goblin {}", e);
        }
    }
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
