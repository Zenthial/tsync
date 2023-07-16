use ssh2::Channel;

pub fn close_channel(channel: &mut Channel) {
    channel.send_eof().unwrap();
    // channel.wait_eof().unwrap();
    channel.close().unwrap();
    // channel.wait_close().unwrap();
}
