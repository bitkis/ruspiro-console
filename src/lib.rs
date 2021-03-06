/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Appache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-console/0.3.1")]
#![cfg_attr(not(any(test, doctest)), no_std)]

//! # Console abstraction
//!
//! This crate provides a console abstraction to enable string output to a configurable output channel.
//! It also provides the convinient macros (``print!`` and ``println!``) to output text that are usually not
//! available in ``[no_std]`` environments. However this crate also provide macros to indicate the severity of the
//! message that shall be printed. Those are ``info!``, ``warn!`` and ``error!``.
//!
//! # Dependencies
//! This crate uses macros to provide formatted strings. This formatting requires a memory allocator to
//! be present (as part of the ``alloc`` crate). So when using this crate provide an allocator such as
//! ``ruspiro_allocator``.
//!
//! # Example
//! To actually set an active output channel you need to provide a structure that implements the ``ConsoleImpl`` trait. This
//! for example is done in the Uart like so:
//! ```ignore
//! impl ConsoleImpl for Uart1 {
//!     fn putc(&self, c: char) {
//!         self.send_char(c);
//!     }
//!
//!     fn puts(&self, s: &str) {
//!         self.send_string(s);
//!     }
//! }
//! ```
//!
//! If this trait has been implemented this structure can be used as actual console. To use it there should be the following
//! code written at the earliest possible point in the main crate of the binary (e.g. the kernel)
//! ```ignore
//! use ruspiro_console::*;
//! use ruspiro_uart::*; // as we demonstrate with the Uart.
//!
//! fn main() {
//!     let mut uart = Uart1::new(); // create a new uart struct
//!     if uart.initialize(250_000_000, 115_200).is_ok() { // initialize the Uart with fixed core rate and baud rate
//!         CONSOLE.take_for(|cons| cons.replace(uart)); // from this point CONSOLE takes ownership of Uart
//!         // uncommenting the following line will give compiler error as uart is moved
//!         // uart.send_string("I'm assigned to a console");
//!     }
//!
//!     // if everything went fine uart should be assigned to the console for generic output
//!     println!("Console is ready and display's through uart");
//! }
//! ```

pub extern crate alloc;

#[macro_use]
pub mod macros;
pub use macros::*;

use alloc::boxed::Box;
use ruspiro_singleton::Singleton;

/// Every "real" console need to implement this trait. Also the explicit Drop trait need to be implemented
/// as the drop method of the implementing console will be called as soon as the actual console does release
/// ownership of it
pub trait ConsoleImpl: Drop {
    /// pass a single character to the output channel
    fn putc(&self, c: char);
    /// pass a string to the output channel
    fn puts(&self, s: &str);
}

/// The Console singleton used by print! and println! macros
pub static CONSOLE: Singleton<Console> = Singleton::<Console>::new(Console {
    current: None,
    default: DefaultConsole {},
});

/// The base printing function hidden behind the print! and println! macro. This function fowards all calls to the
/// generic console which puts the string to the assigned output channel.
pub fn print(s: &str) {
    // pass the string to the actual configured console to be printed
    CONSOLE.use_for(|console| {
        console.get_current().puts(s);
    });
}

/// The representation of the abstract console
pub struct Console {
    current: Option<Box<dyn ConsoleImpl>>,
    default: DefaultConsole,
}

impl Console {
    /// Retrieve the current active console to be used for passing strings to to get printend somewhere
    pub fn get_current(&self) -> &dyn ConsoleImpl {
        if let Some(ref console) = self.current {
            console.as_ref()
        } else {
            &self.default
        }
    }

    /// Replacing the current active console. Once the new has been set the [drop] function of the previous one is
    /// called. The Console takes ownership of the active once. Access to the active console outside the abstraction
    /// is not possible and should not be.
    pub fn replace<T: ConsoleImpl + 'static>(&mut self, console: T) {
        self.current.replace(Box::from(console));
    }
}

/// The default console is a kind of fall back that prints nothing...
struct DefaultConsole;

impl ConsoleImpl for DefaultConsole {
    fn putc(&self, _: char) {
        // the default console does nothing as it is not linked to any hardware
    }

    fn puts(&self, _: &str) {
        // the default console does nothing as it is not linked to any hardware
    }
}

impl Drop for DefaultConsole {
    fn drop(&mut self) {
        // the default console has no resources that need to be freed while dropping
    }
}
