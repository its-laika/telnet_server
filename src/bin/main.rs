use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener};
use std::thread;
use telnet_server::telnet::{TelnetSession, TelnetSessionConfig};

const BIND_ADDRESS: &str = "127.0.0.1:9000";
const MAX_MESSAGE_SIZE: usize = 4096;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(BIND_ADDRESS)?;

    for stream in listener.incoming() {
        thread::spawn(move || {
            let mut stream = match stream {
                Ok(s) => s,
                Err(_) => {
                    /* Stream not available. Just drop this client. */
                    return;
                }
            };

            let session_config = TelnetSessionConfig {
                handle_ansi_escape_sequences: false,
            };
            let mut telnet_session = TelnetSession::create(&session_config);
            let mut buffer: [u8; MAX_MESSAGE_SIZE] = [0; MAX_MESSAGE_SIZE];
            let mut response = vec![];

            loop {
                /* Try loading next client message / command */
                let read_bytes = match stream.read(&mut buffer) {
                    Ok(0) => {
                        /* Connection closed. Shutdown may fail but we'll ignore that as
                         * the client is dropped anyway. */
                        stream.shutdown(Shutdown::Both).unwrap_or_default();
                        return;
                    }
                    Ok(c) => c,
                    Err(_) => {
                        /* Stream not available. Just drop this client. */
                        return;
                    }
                };

                response.clear();

                if let Some(telnet_data) = telnet_session.accept_data(&buffer[..read_bytes]) {
                    response.extend_from_slice(telnet_data.as_slice());
                }

                if let Some(message_response) = generate_message_response(&mut telnet_session) {
                    response.extend_from_slice(message_response.as_slice());
                }

                if !response.is_empty() && stream.write_all(response.as_slice()).is_err() {
                    /* Stream not available. Just drop this client. */
                    return;
                }
            }
        });
    }

    Ok(())
}

fn generate_message_response(telnet_session: &mut TelnetSession) -> Option<Vec<u8>> {
    let message = telnet_session
        .get_data_buffer()
        .iter()
        .map(|&c| c as u8)
        .collect::<Vec<u8>>();

    if let Some(&_last @ b'\n') = message.last() {
        telnet_session.clear_data_buffer();
        return Some(["You sent: ".as_bytes(), &message, "\r\n".as_bytes()].concat());
    }

    None
}
