use rpassword::prompt_password;
use ssh2::{KeyboardInteractivePrompt, Session};

enum Method {
    Password,
    Keyboard,
}

struct Prompt;

impl KeyboardInteractivePrompt for Prompt {
    fn prompt(
        &mut self,
        _username: &str,
        _instructions: &str,
        prompts: &[ssh2::Prompt<'_>],
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

fn parse_methods(methods: &str) -> Method {
    if methods.contains("password") {
        return Method::Password;
    } else if methods.contains("keyboard") {
        return Method::Keyboard;
    } else {
        unreachable!()
    }
}

fn prompt_user_for_pass() -> String {
    prompt_password("Password:").unwrap()
}

pub fn authenticate(sess: &mut Session, username: &str) -> Result<(), ssh2::Error> {
    let methods = sess.auth_methods(username).unwrap();
    let method = parse_methods(methods);

    match method {
        Method::Password => sess.userauth_password(username, &prompt_user_for_pass()),
        Method::Keyboard => {
            let mut p = Prompt {};
            sess.userauth_keyboard_interactive(username, &mut p)
        }
    }
}
