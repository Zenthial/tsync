use ssh2::Channel;

pub fn close_channel(channel: &mut Channel) {
    if let Err(e) = channel.send_eof() {
        panic!("failed to send EOF to channel\n  message: {}", e.message());
    }
    // channel.wait_eof().unwrap();
    if let Err(e) = channel.close() {
        panic!("failed to close channel\n  message: {}", e.message());
    }
    // channel.wait_close().unwrap();
}
