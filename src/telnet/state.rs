use crate::iter::contains_sequence;
use std::{
    cmp::min,
    io::{Error, Read},
};

const ECHO: u8 = 1;
const ERASE_LINE: u8 = 248;

const BEL: u8 = 7;

const CHAR_BACK_SPACE: u8 = 8;
const CHAR_ESCAPE: u8 = 27;
const CHAR_DELETE: u8 = 127;
const CHAR_ERASE: u8 = 247;

const IAC: u8 = 255;
/// "IAC SB"
const IAC_SUBNEGOTIATION_START: u8 = 250;
/// "IAC SE"
const IAC_SUBNEGOTIATION_END: u8 = 240;
const IAC_WILL: u8 = 251;
const IAC_WONT: u8 = 252;
const IAC_DO: u8 = 253;
const IAC_DONT: u8 = 254;

/// Sequence for erasing current line in ANSI terminals
const ANSI_SEQUENCE_ERASE_LINE: [u8; 5] = [CHAR_ESCAPE, 91, 50, 75, 13];

/// Any known ending character of ANSI escape sequences
const CHARS_ESCAPE_SEQUENCE_END: [char; 20] = [
    'A', /* CUU */
    'B', /* CUD */
    'C', /* CUF */
    'D', /* CUB */
    'E', /* CNL */
    'F', /* CPL */
    'G', /* CHA */
    'H', /* CUP */
    'J', /* ED */
    'K', /* EL */
    'S', /* SU */
    'T', /* SD */
    'f', /* HVP */
    'm', /* SGR */
    'i', /* AUX */
    'n', /* DSR */
    's', /* SCP, SCOSC */
    'u', /* RCP, SCORC */
    'h', /* DECTCEM */
    'l', /* DECTCEM */
];

const CHARS_LINE_BREAK: [u8; 2] = [b'\r', b'\n'];

/// Type for read-only bytes
pub type Bytes = Box<[u8]>;
pub type BytesResult = Result<Option<Bytes>, Error>;

/// Struct that holds and handles the current state of a TELNET session.
/// Implements [`Read`] to get the handled readable, non-command data and a fake
/// `Write` to accept incoming TCP data.
///
/// # Notice
///
/// This struct is independent of any service related input or TCP at all. It is
/// just a bare state machine.
pub struct State {
    /// Buffer of "readable" received data that can be obtained by using the
    /// [`Read`] trait
    output_buffer: Vec<u8>,
    /// Current overall mode
    mode: Mode,
    /// Indicates whether every incoming, non-command char should be echoed back
    /// to the connection
    is_echoing: bool,
    /// If true, ANSI escape sequences will be handled like normal non-command
    /// input. Otherwise, sequences will be ignored and a BEL is sent back to
    /// notice.
    handle_ansi_escape_sequences: bool,
}

/// Configuration to set up a new [`State`]
#[derive(Default)]
pub struct StateConfig {
    /// If true, ANSI escape sequences will be handled like normal non-command
    /// input. Otherwise, sequences will be ignored and a BEL is sent back to
    /// notice.
    pub handle_ansi_escape_sequences: bool,
}

/// Enumeration of overall modes that a TELNET state may have
enum Mode {
    /// Incoming, non-command data (e.g. text)
    Idle,
    /// Incoming command data (e.g. WILL, WONT, DO, DONT)
    Command,
    /// Incoming command data for WILL command
    CommandWill,
    /// Incoming command data for WONT command
    CommandWont,
    /// Incoming command data for DO command
    CommandDo,
    /// Incoming command data for DONT command
    CommandDont,
    /// Incoming command data for sub negotiation command
    SubNegotiation,
    /// Incoming escape sequence. This is not a "real" mode but we need it as
    /// you can choose to ignore ANSI escape sequences because it doesn't really
    /// make sense to evaluate these.
    AnsiEscapeSequence,
}

impl State {
    /// Creates a new [`State`] with given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - [`StateConfig`] to set up the behaviour of the resutlting
    ///   [`State`]
    ///
    /// # Returns
    ///
    /// Fresh [`State`] for a TELNET session
    ///
    /// # Examples
    ///
    /// ```rust
    /// use telnet_server::telnet::{StateConfig, State};
    ///
    /// let config = StateConfig::default();
    /// let state = State::new(&config);
    /// ```
    pub fn new(config: &StateConfig) -> Self {
        State {
            output_buffer: vec![],
            mode: Mode::Idle,
            is_echoing: false,
            handle_ansi_escape_sequences: config.handle_ansi_escape_sequences,
        }
    }

    /// Writes a buffer of incoming TELNET data into the state, returning an
    /// immediate response to the other part. Consumes the whole buffer _unless_
    /// there's an error (e.g. invalid data) which immediatelly results in an
    /// [`Error`].
    ///
    /// # Arguments
    ///
    /// * `data` - Incoming TELNET data
    ///
    /// # Returns
    ///
    /// * `Err(std::io::Error)` if an error occurs. In this case the internal
    ///   TELNET state possibly mismatches the "real" state. This _should_ lead
    ///   to the termination of the TELNET session at all.
    /// * `Ok(Some([u8]))` if data should be sent back
    /// * `Ok(None)` if everythings fine
    ///
    /// # Examples
    ///
    /// ```rust
    /// use telnet_server::telnet::{StateConfig, State};
    ///
    /// let config = StateConfig::default();
    /// let mut state = State::new(&config);
    ///
    /// let data = [255, 253, 1];
    /// let result = state.write(&data)?;
    ///
    /// assert!(result.is_some());
    /// let result = result.unwrap();
    ///
    /// // write back result to TCP connection...
    ///
    /// Ok::<(), std::io::Error>(())
    /// ```
    pub fn write(&mut self, buf: &[u8]) -> BytesResult {
        let mut response: Vec<u8> = vec![];

        for &next in buf {
            let result = match self.mode {
                Mode::Idle => self.next_on_idle(next),
                Mode::Command => self.next_as_command(next),
                Mode::CommandWill => self.next_as_will(next),
                Mode::CommandWont => self.next_as_wont(next),
                Mode::CommandDo => self.next_as_do(next),
                Mode::CommandDont => self.next_as_dont(next),
                Mode::SubNegotiation => self.next_as_sub_negotiation(next),
                Mode::AnsiEscapeSequence => self.next_as_escape_sequence(next),
            };

            if let Ok(Some(v)) = result {
                response.extend_from_slice(&v);
            } else if result.is_err() {
                return result;
            }
        }

        if !response.is_empty() {
            Ok(Some(response.into_boxed_slice()))
        } else {
            Ok(None)
        }
    }

    /// Handles incoming `next` byte when [`State`] is in idle mode
    ///
    /// # Returns
    ///
    /// * `Ok(None)` - Everythings okay, no need to write something back
    /// * `Ok(Some(Bytes))` - Everythings okay, something has to be written back
    /// * `Err` - Data could not be interpreted
    fn next_on_idle(&mut self, next: u8) -> BytesResult {
        match next {
            IAC => self.mode = Mode::Command,
            CHAR_DELETE | CHAR_BACK_SPACE | CHAR_ERASE => {
                self.output_buffer.pop();

                if self.is_echoing {
                    /* Return fake backspace on echo mode */
                    return Ok(Some(Box::new([CHAR_BACK_SPACE, b' ', CHAR_BACK_SPACE])));
                }
            }
            ERASE_LINE => {
                Self::erase_current_line(&mut self.output_buffer);

                if self.is_echoing {
                    return Ok(Some(ANSI_SEQUENCE_ERASE_LINE.into()));
                }

                return Ok(None);
            }
            CHAR_ESCAPE => {
                self.mode = Mode::AnsiEscapeSequence;

                if self.is_echoing {
                    return Ok(Some(Box::new([next])));
                }

                if self.handle_ansi_escape_sequences {
                    self.output_buffer.push(next);
                }
            }
            _ => {
                self.output_buffer.push(next);

                if self.is_echoing {
                    return Ok(Some(Box::new([next])));
                }
            }
        }

        Ok(None)
    }

    /// Handles incoming `next` byte when [`State`] is in IAC mode
    ///
    /// # Returns
    ///
    /// * `Ok(None)` - Everythings okay, no need to write something back
    /// * `Ok(Some(Bytes))` - Everythings okay, something has to be written back
    /// * `Err` - Data could not be interpreted
    fn next_as_command(&mut self, next: u8) -> BytesResult {
        match next {
            IAC_WILL => self.mode = Mode::CommandWill,
            IAC_WONT => self.mode = Mode::CommandWont,
            IAC_DO => self.mode = Mode::CommandDo,
            IAC_DONT => self.mode = Mode::CommandDont,
            IAC_SUBNEGOTIATION_START => self.mode = Mode::SubNegotiation,
            _ => {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unknown command '{next}'"),
                ))
            }
        };

        Ok(None)
    }

    /// Handles incoming `next` byte when [`State`] is in IAC WILL mode
    ///
    /// # Returns
    ///
    /// * `Ok(None)` - Everythings okay, no need to write something back
    /// * `Ok(Some(Bytes))` - Everythings okay, something has to be written back
    /// * `Err` - Data could not be interpreted
    fn next_as_will(&mut self, _next: u8) -> BytesResult {
        /* Ignore message, just go back to idle state */
        self.mode = Mode::Idle;
        Ok(None)
    }

    /// Handles incoming `next` byte when [`State`] is in IAC WONT mode
    ///
    /// # Returns
    ///
    /// * `Ok(None)` - Everythings okay, no need to write something back
    /// * `Ok(Some(Bytes))` - Everythings okay, something has to be written back
    /// * `Err` - Data could not be interpreted
    fn next_as_wont(&mut self, _next: u8) -> BytesResult {
        /* Ignore message, just go back to idle state */
        self.mode = Mode::Idle;
        Ok(None)
    }

    /// Handles incoming `next` byte when [`State`] is in IAC DO mode
    ///
    /// # Returns
    ///
    /// * `Ok(None)` - Everythings okay, no need to write something back
    /// * `Ok(Some(Bytes))` - Everythings okay, something has to be written back
    /// * `Err` - Data could not be interpreted
    fn next_as_do(&mut self, next: u8) -> BytesResult {
        self.mode = Mode::Idle;

        if next == ECHO {
            self.is_echoing = true;
            return Ok(Some(Box::new([IAC, IAC_WILL, ECHO])));
        }

        /* Whatever they're asking for, we're not supporting it probably. */
        Ok(Some(Box::new([IAC, IAC_WONT, next])))
    }

    /// Handles incoming `next` byte when [`State`] is in IAC DONT mode
    ///
    /// # Returns
    ///
    /// * `Ok(None)` - Everythings okay, no need to write something back
    /// * `Ok(Some(Bytes))` - Everythings okay, something has to be written back
    /// * `Err` - Data could not be interpreted
    fn next_as_dont(&mut self, next: u8) -> BytesResult {
        self.mode = Mode::Idle;

        if next == ECHO {
            self.is_echoing = false;
        }

        /* Whatever they're asking for, we're not supporting it probably.
         * So it's fine to say that we won't do it. */
        Ok(Some(Box::new([IAC, IAC_WONT, next])))
    }

    /// Handles incoming `next` byte when [`State`] is in IAC SB mode
    ///
    /// # Returns
    ///
    /// * `Ok(None)` - Everythings okay, no need to write something back
    /// * `Ok(Some(Bytes))` - Everythings okay, something has to be written back
    /// * `Err` - Data could not be interpreted
    fn next_as_sub_negotiation(&mut self, next: u8) -> BytesResult {
        /* We're NOT handling sub negotiations right now. */
        if next == IAC_SUBNEGOTIATION_END {
            self.mode = Mode::Idle;
        }

        Ok(None)
    }

    /// Handles incoming `next` byte when [`State`] is in idle mode and in an
    /// ANSI escape sequence
    ///
    /// # Returns
    ///
    /// * `Ok(None)` - Everythings okay, no need to write something back
    /// * `Ok(Some(Bytes))` - Everythings okay, something has to be written back
    /// * `Err` - Data could not be interpreted
    fn next_as_escape_sequence(&mut self, next: u8) -> BytesResult {
        if self.handle_ansi_escape_sequences {
            self.output_buffer.push(next);

            if CHARS_ESCAPE_SEQUENCE_END.contains(&(next as char)) {
                self.mode = Mode::Idle;
            }

            if self.is_echoing {
                Ok(Some(Box::new([next])))
            } else {
                Ok(None)
            }
        } else {
            if !CHARS_ESCAPE_SEQUENCE_END.contains(&(next as char)) {
                return Ok(None);
            }

            self.mode = Mode::Idle;

            Ok(Some(Box::new([BEL])))
        }
    }

    /// Erases the current line from given text buffer. According to
    /// [RFC-854](https://www.rfc-editor.org/rfc/rfc854#page-13), the last
    /// CR LF should be kept.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Text buffer that should be updated. All current line
    ///   characters will be removed from the [`Vec`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use telnet_server::telnet::State;
    ///
    /// let mut buffer = vec![b'a', b'b', b'c', b'\r', b'\n', b'd', b'e', b'f'];
    /// State::erase_current_line(&mut buffer);
    /// assert_eq!(buffer, [b'a', b'b', b'c', b'\r', b'\n']);
    ///
    /// State::erase_current_line(&mut buffer);
    /// assert_eq!(buffer, [b'a', b'b', b'c', b'\r', b'\n']);
    ///
    /// let mut buffer = vec![b'a', b'b', b'c', b'd', b'e', b'f'];
    /// State::erase_current_line(&mut buffer);
    /// assert!(buffer.is_empty());
    /// ```
    fn erase_current_line(buffer: &mut Vec<u8>) {
        loop {
            let buffer_len = buffer.len();

            /* Remove all chars until \r\n reached */
            if buffer_len < 2 {
                buffer.clear();
                break;
            }

            let start_index = buffer_len - 2;
            if contains_sequence(&buffer[start_index..], &CHARS_LINE_BREAK) {
                break;
            }

            buffer.pop();
        }
    }
}

impl Read for State {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let limit = min(buf.len(), self.output_buffer.len());

        let (left, _) = buf.split_at_mut(limit);
        left.copy_from_slice(&self.output_buffer[..limit]);
        self.output_buffer.drain(..limit);

        Ok(limit)
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn erase_current_line_should_work() {
        let mut buffer = vec![b'a', b'b', b'c', b'\r', b'\n', b'd', b'e', b'f'];
        State::erase_current_line(&mut buffer);
        /* RFC-854: 'The recipient should delete characters from the data stream
         * back to, but not including, the last "CR LF" sequence sent over the
         * TELNET connection.' */
        assert_eq!(buffer, [b'a', b'b', b'c', b'\r', b'\n']);

        State::erase_current_line(&mut buffer);
        assert_eq!(buffer, [b'a', b'b', b'c', b'\r', b'\n']);

        let mut buffer = vec![b'a', b'b', b'c', b'd', b'e', b'f'];
        State::erase_current_line(&mut buffer);
        assert!(buffer.is_empty());
    }
}
