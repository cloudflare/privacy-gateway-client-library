use std::{cell::RefCell, error::Error, slice};

use libc::{c_char, c_int};
use log::{error, debug};

use env_logger::{Builder, Target};

thread_local! {
    static LAST_ERROR: RefCell<Option<Box<dyn Error>>> = RefCell::new(None);
}

#[no_mangle]
pub extern "C" fn initialize_logging() {
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);

    builder.init();
    debug!("Logger initialized");
}

/// Update the last error, clearing the old one.
pub fn update_last_error<E: Error + 'static>(err: E) {
    error!("Setting last error {err}");
    {
        let mut cause = err.source();
        while let Some(parent_err) = cause {
            error!("Caused by: {parent_err}");
            cause = parent_err.source();
        }
    }
    LAST_ERROR.with(|prev| {
        *prev.borrow_mut() = Some(Box::new(err));
    });
}

/// Retrieve the most recent error, clearing it in the process.
pub fn take_last_error() -> Option<Box<dyn Error>> {
    LAST_ERROR.with(|prev| prev.borrow_mut().take())
}

/// Return the number of bytes in the last error message.
/// Does not include any trailing null terminators.
#[no_mangle]
pub extern "C" fn last_error_length() -> libc::c_int {
    LAST_ERROR.with(|prev| match *prev.borrow() {
        Some(ref err) => err.to_string().len() as libc::c_int,
        None => 0,
    })
}

/// Write the most recent error UTF-8 encoded message into a provided buffer
///
/// If there are no recent errors then this returns 0. -1 is returned if there is an error but something bad happened:
/// - provided `buffer` is too small
/// - or a provided `buffer` is a null pointer
///
/// Otherwise the function returns the number of bytes written to the buffer.
///
/// # Safety
/// The invariants are described here [`from_raw_parts_mut`](std::slice::from_raw_parts_mut#safety)
pub unsafe extern "C" fn last_error_message(buffer: *mut c_char, length: c_int) -> c_int {
    if buffer.is_null() {
        error!("Null pointer passed into last_error_message() as the buffer");
        return -1;
    }

    let last_error = match take_last_error() {
        Some(err) => err,
        None => return 0,
    };

    let error_message = last_error.to_string(); 

    let buffer = slice::from_raw_parts_mut(buffer as *mut u8, length as usize);

    if error_message.len() >= buffer.len() {
        error!("Buffer providded for writing last message is to small!");
        error!(
            "Expected at least {} bytes but got {}",
            error_message.len() + 1,
            buffer.len()
        );
        return -1;
    }

    std::ptr::copy_nonoverlapping(
        error_message.as_ptr(),
        buffer.as_mut_ptr(),
        error_message.len(),
    );

    // Add a trailling null terminator
    buffer[error_message.len()] = 0;
    error_message.len() as c_int
}
