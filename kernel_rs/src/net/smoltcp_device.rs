//! smoltcp device adapter for e1000
//! 
//! Implements smoltcp::phy::Device trait for e1000 network card

use smoltcp::phy::{self, Device, DeviceCapabilities};
use smoltcp::time::Instant;
use alloc::vec::Vec;

/// Device adapter for e1000
pub struct E1000Adapter;

impl E1000Adapter {
    pub fn new() -> Self {
        E1000Adapter
    }
}

impl Device for E1000Adapter {
    type RxToken<'a> = E1000RxToken where Self: 'a;
    type TxToken<'a> = E1000TxToken where Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        // Try to receive a packet
        if let Some(buffer) = super::device::recv_packet() {
            Some((
                E1000RxToken { buffer },
                E1000TxToken,
            ))
        } else {
            None
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        // TX is always available (with queue)
        Some(E1000TxToken)
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1500;
        caps.max_burst_size = Some(256);
        caps
    }
}

/// RX Token for received packets
pub struct E1000RxToken {
    buffer: Vec<u8>,
}

impl phy::RxToken for E1000RxToken {
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        f(&mut self.buffer)
    }
}

/// TX Token for transmitting packets
pub struct E1000TxToken;

impl phy::TxToken for E1000TxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buffer = vec![0u8; len];
        let result = f(&mut buffer);
        
        // Send the packet
        let _ = super::device::send_packet(&buffer);
        
        result
    }
}
