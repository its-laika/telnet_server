use super::State;
use crate::read;
use std::{
    io::{self, ErrorKind, Read, Result, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
};

/// Handles the TCP connection for a TELNET service, allowing reading and
/// writing access while also handling the internal TELNET state.
///
/// Implements [`std::io::Read`] and [`std::io::Write`] to receive and send
/// messages from/to the connection.
#[derive(Clone)]
pub struct Session {
    /// Reference to a TELNET connection [`State`]
    state: Arc<Mutex<State>>,
    /// Refence to the TCP connection
    tcp_stream: Arc<Mutex<TcpStream>>,
}

impl Session {
    /// Creates new [`Session`] based on given [`TcpStream`] and a fresh
    /// [`State`].
    /// Also ensures that the TCP stream is non-blocking as otherwise the
    /// session becomes unusable.
    ///
    /// # Arguments
    ///
    /// * `state` - A fresh [`State`]
    /// * `tcp_stream` - [`TcpStream`] for a TELNET based session. Notice that
    ///   there's no instant check if this is for TELNET or anthother protocol.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` on success
    /// * `Err(std::io::Error)` if `tcp_stream` cannot be set to non-blocking
    pub fn new(state: State, tcp_stream: TcpStream) -> Result<Self> {
        tcp_stream.set_nonblocking(true)?;

        Ok(Self {
            state: Arc::new(Mutex::new(state)),
            tcp_stream: Arc::new(Mutex::new(tcp_stream)),
        })
    }

    /// Listens to and handles incoming TCP data.
    /// Should be called in a background thread as it blocks. As the internal
    /// TCP stream is set to non-blocking, reading and writing on a cloned
    /// [`Session`] is still possible.
    ///
    /// # Returns
    ///
    /// Only returns an `Err(std::io::Error)` on TCP errors as it runs
    /// indefinitely.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::thread;
    /// use telnet_server::telnet::{Session, State, StateConfig};
    ///
    /// let mut session = Session::new(State::new(&StateConfig::default()), tcp_stream)?;
    ///
    /// let session_listen = session.clone();
    /// let handle = thread::spawn(move || session_listen.listen());
    ///
    /// // Send and receive messages here...
    ///
    /// handle.join().expect("Should await thread");
    ///
    /// Ok(())
    /// ```
    pub fn listen(self) -> Result<()> {
        let mut buf: [u8; 255] = [0; 255];

        loop {
            let mut tcp_stream = match self.tcp_stream.try_lock() {
                Ok(t) => t,
                Err(_) => continue,
            };

            let tcp_data = match tcp_stream.read(&mut buf) {
                Ok(read_bytes) => &buf[..read_bytes],
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        continue;
                    }

                    return Err(e);
                }
            };

            if let Some(telnet_data) = self
                .state
                .lock()
                .expect("Should lock state")
                .write(tcp_data)?
            {
                tcp_stream.write_all(&telnet_data)?;
                tcp_stream.flush()?;
            }
        }
    }
}

impl io::Write for Session {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.tcp_stream
            .lock()
            .expect("Should lock stream")
            .write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.tcp_stream.lock().expect("Should lock stream").flush()
    }
}

impl io::Read for Session {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.state.lock().expect("Should lock state").read(buf)
    }
}

impl read::Read for Session {
    fn read_line_waiting(&mut self) -> Result<String> {
        let mut line = String::new();
        let mut buf: [u8; 1] = [0];

        loop {
            match self.read(&mut buf) {
                Ok(1) => {
                    let next = buf[0] as char;
                    line.push(next);
                    if next == '\n' {
                        break;
                    }
                }
                Ok(0) => {
                    continue;
                }
                Ok(_) => panic!("Out of range"),
                Err(e) => return Err(e),
            };
        }

        Ok(line)
    }
}
