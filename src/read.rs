//! Module containing resources for reading data.
//! The reason for this module to exist is "missing" - but mandatory -
//! functionality in the [`std::io::Read`] trait.

use std::io::Error;

/// Trait for extending [`std::io::Read`] to add "missing" functionality
pub trait Read {
    /// Reads line to a [`String`], ensuring it is never empty.
    /// Spins until a `\n` has been found, so that - even if the buffer is empty
    /// at some point - the result is always a non-empty line.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` with a non-empty string, ending with `\n`
    /// * `Err(std::io::Error)` if reading fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use telnet_server::telnet::{Session, State, StateConfig};
    /// use crate::telnet_server::read::Read;
    ///
    /// let mut session = Session::new(State::new(&StateConfig::default()), tcp_stream)?;
    ///
    /// // set up session to receive data...
    ///
    /// let incoming = session.read_line_waiting()?;
    ///
    /// assert!(incoming.len() > 0);
    /// assert!(incoming.ends_with('\n'));
    ///
    /// Ok(())
    /// ```
    fn read_line_waiting(&mut self) -> Result<String, Error>;
}
