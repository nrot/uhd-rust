use std::marker::PhantomData;
use std::ptr;

use crate::TransmitMetadata;
use crate::error::{check_status, Error};
use crate::receive_metadata::ReceiveMetadata;
use crate::stream::StreamCommand;
use crate::usrp::Usrp;
use std::os::raw::c_void;

/// A streamer used to receive samples from a USRP
///
/// The type parameter I is the type of sample that this streamer receives.
#[derive(Debug)]
pub struct TransmitStreamer<I> {
    /// Streamer handle
    handle: uhd_sys::uhd_tx_streamer_handle,
    /// A vector of pointers to buffers (used in transmit() to convert `&mut [&mut [I]]` to `*mut *mut I`
    /// without reallocating memory each time
    ///
    /// Invariant: If this is not empty, its length is equal to the value returned by
    /// self.num_channels().
    buffer_pointers: Vec<*mut c_void>,
    /// Link to the USRP that this streamer is associated with
    // usrp: PhantomData<&'usrp Usrp>,
    /// Item type phantom data
    item_phantom: PhantomData<I>,
}


impl<I> TransmitStreamer< I> {
    /// Creates a receive streamer with a null streamer handle (for internal use only)
    ///
    /// After creating a streamer with this function, its streamer handle must be initialized.
    pub(crate) fn new(capacity: usize) -> Self {
        TransmitStreamer {
            handle: ptr::null_mut(),
            buffer_pointers: Vec::with_capacity(capacity),
            // usrp: PhantomData,
            item_phantom: PhantomData,
        }
    }

    pub fn buff_size(&self)->usize{
        self.buffer_pointers.capacity()
    }

    pub(crate) fn set_len(&mut self, cap: usize){
        unsafe {self.buffer_pointers.set_len(cap)};
    }

    pub fn buff_len(&self)->usize{
        self.buffer_pointers.len()
    }

    /// Returns a reference to the streamer handle
    pub(crate) fn handle_mut(&mut self) -> &mut uhd_sys::uhd_tx_streamer_handle {
        &mut self.handle
    }
    /// Returns the streamer handle
    pub(crate) fn handle(&mut self) -> uhd_sys::uhd_tx_streamer_handle {
        self.handle
    }

    /// Returns the number of channels that this streamer is associated with
    pub fn num_channels(&self) -> usize {
        let mut num_channels = 0usize;
        check_status(unsafe {
            uhd_sys::uhd_tx_streamer_num_channels(
                self.handle,
                &mut num_channels as *mut usize as *mut _,
            )
        })
        .unwrap();
        num_channels
    }

    /// Receives samples from the USRP
    ///
    /// buffers: One or more buffers (one per channel) where the samples will be written. All
    /// buffers should have the same length. This function will panic if the number of buffers is
    /// not equal to self.num_channels(), or if not all buffers have the same length.
    ///
    /// timeout: The timeout for the receive operation, in seconds
    ///
    /// one_packet: If this is true, one call to receive() will not copy samples from more than
    /// one packet of the underlying protocol
    ///
    /// On success, this function returns a ReceiveMetadata object with information about
    /// the number of samples actually received.
    pub fn send(
        &mut self,
        buffers: &mut [&mut [I]],
        timeout: f64,
    ) -> Result<(), Error> {
        let mut metadata = TransmitMetadata::default();
        let mut samples_received = 0usize;

        // Initialize buffer_pointers
        if self.buffer_pointers.is_empty() {
            self.buffer_pointers
                .resize(self.num_channels(), ptr::null_mut());
        }
        // Now buffer_pointers.len() is equal to self.num_channels().
        assert_eq!(
            buffers[0].len(),
            self.buffer_pointers.len(),
            "Number of buffers is not equal to this streamer's number of channels"
        );
        // Check that all buffers have the same length
        let buffer_length = check_equal_buffer_lengths(buffers);

        // Copy buffer pointers into C-compatible form
        for (entry, buffer) in self.buffer_pointers.iter_mut().zip(buffers.iter_mut()) {
            *entry = buffer.as_mut_ptr() as *mut c_void;
        }

        check_status(unsafe {
            uhd_sys::uhd_tx_streamer_send(
                self.handle,
                self.buffer_pointers.as_mut_ptr() as *mut *const _,
                buffer_length as _,
                metadata.handle_mut(),
                timeout,
                &mut samples_received as *mut usize as *mut _,
            )
        })?;
        metadata.set_samples(samples_received);

        Ok(())
    }

}

/// Checks that all provided buffers have the same length. Returns the length of the buffers,
/// or 0 if there are no buffers. Panics if the buffer lengths are not equal.
fn check_equal_buffer_lengths<I>(buffers: &mut [&mut [I]]) -> usize {
    buffers
        .iter()
        .fold(None, |prev_size, buffer| {
            match prev_size {
                None => {
                    // Store the size of the first buffer
                    Some(buffer.len())
                }
                Some(prev_size) => {
                    assert_eq!(prev_size, buffer.len(), "Unequal buffer sizes");
                    Some(prev_size)
                }
            }
        })
        .unwrap_or(0)
}

impl<I> Drop for TransmitStreamer< I> {
    fn drop(&mut self) {
        let _ = unsafe { uhd_sys::uhd_tx_streamer_free(&mut self.handle) };
    }
}

// Thread safety: see https://files.ettus.com/manual/page_general.html#general_threading
// All functions are thread-safe, except that the uhd_tx_streamer send(), uhd_tx_streamer recv(), and
// uhd_tx_streamer recv_async_msg() functions. The corresponding Rust wrapper functions take &mut
// self, which enforces single-thread access.
unsafe impl<I> Send for TransmitStreamer< I> {}
unsafe impl<I> Sync for TransmitStreamer< I> {}
