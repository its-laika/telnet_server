//! TELNET module
//!
//! This module contains state and session handling for TCP connections to a
//! TELNET service.
//! It has two sub modules:
//! * [`Session`] handles the TCP connection
//!   * [`State`] handles the internal (TELNET) state of an existing connection
//!     * [`StateConfig`] can be used to configure the handling of the [`State`]
//!       in specific cases.
pub mod session;
pub mod state;

pub use session::Session;
pub use state::{State, StateConfig};
