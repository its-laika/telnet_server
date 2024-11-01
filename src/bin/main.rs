use std::io::{Error, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use telnet_server::read::Read;
use telnet_server::telnet::{Session, State, StateConfig};

const BIND_ADDRESS: &str = "127.0.0.1:9000";

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(BIND_ADDRESS)?;

    for stream in listener.incoming() {
        thread::spawn(move || {
            if let Ok(stream) = stream {
                let _ = handle_connection(stream);
            };
        });
    }

    Ok(())
}

fn handle_connection(tcp_stream: TcpStream) -> Result<(), Error> {
    // Set up State and Session
    let state_config = StateConfig::default();
    let state = State::new(&state_config);
    let mut session = Session::new(state, tcp_stream)?;

    // Make the Session listen to incoming TCP data in the background
    let session_listen = session.clone();
    let handle = thread::spawn(move || session_listen.listen());

    loop {
        // Handle incoming TELNET messages:
        let incoming = session.read_line_waiting()?;
        let answer = format!("You sent: {incoming}");

        if session.write_all(answer.as_bytes()).is_err() {
            break;
        }

        if session.flush().is_err() {
            break;
        }
    }

    handle.join().expect("Should await thread")
}
