/// GATT service definition for BLE messaging
///
/// This module provides GATT service abstractions with characteristic-based messaging,
/// fragmentation, reassembly, and write queue management with backpressure.
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use thiserror::Error;

/// Maximum GATT characteristic write size (protocol limitation)
pub const MAX_CHARACTERISTIC_SIZE: usize = 512;

/// Upper bound on the number of fragments a single GATT message may declare.
///
/// A real envelope is at most a few KB (the Drift transport caps payloads at
/// [`crate::drift::frame::FRAME_MAX_PAYLOAD`], 64 KB), and each fragment carries
/// `MAX_CHARACTERISTIC_SIZE - 4` = 508 payload bytes, so a legitimate stream can
/// never exceed ~130 fragments. Treating any larger `total_fragments` as a header
/// lets an *unfragmented* envelope whose first four bytes happen to read as a
/// plausible `[total][index]` pair be misclassified as a fragment stream. That
/// misroute buffers a bogus stream that never completes (the real bytes are
/// consumed as a fragment) and produces truncated-decode / signature-failure
/// noise on `ble_gatt_ingress`. Rejecting impossible totals keeps the
/// classification unambiguous: only a genuine sender fragment can declare a count
/// this small. This bound is intentionally generous (512 >> 130) so no real
/// sender is affected.
pub const MAX_GATT_FRAGMENTS: usize = 512;

/// Default maximum outstanding writes before backpressure
pub const DEFAULT_MAX_OUTSTANDING_WRITES: usize = 10;

/// GATT characteristic types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GattCharacteristic {
    /// Write characteristic for sending data
    Write,
    /// Notify characteristic for receiving data
    Notify,
    /// Status characteristic for connection state
    Status,
}

impl GattCharacteristic {
    /// Get characteristic UUID (short form)
    pub fn uuid(&self) -> u16 {
        match self {
            GattCharacteristic::Write => 0xDF02,
            GattCharacteristic::Notify => 0xDF03,
            GattCharacteristic::Status => 0xDF04,
        }
    }
}

/// Errors for GATT operations
#[derive(Error, Debug, Clone)]
pub enum GattError {
    #[error("Write queue full (backpressure)")]
    WriteQueueFull,
    #[error("Write failed: {0}")]
    WriteFailed(String),
    #[error("Read failed: {0}")]
    ReadFailed(String),
    #[error("Fragmentation error: {0}")]
    FragmentationError(String),
    #[error("Reassembly error: {0}")]
    ReassemblyError(String),
    #[error("Invalid characteristic")]
    InvalidCharacteristic,
    #[error("Not connected")]
    NotConnected,
}

/// Fragment header for GATT messages: [total_fragments: u16 | fragment_index: u16 | data...]
#[derive(Debug, Clone)]
pub struct GattFragmentHeader {
    /// Total number of fragments
    pub total_fragments: u16,
    /// Index of this fragment (0-based)
    pub fragment_index: u16,
}

impl GattFragmentHeader {
    /// Size of fragment header in bytes
    const HEADER_SIZE: usize = 4;

    /// Create a new fragment header
    pub fn new(total_fragments: u16, fragment_index: u16) -> Result<Self, GattError> {
        if total_fragments == 0 || total_fragments as usize > MAX_GATT_FRAGMENTS {
            return Err(GattError::FragmentationError(format!(
                "Fragment total out of plausible range: {total_fragments} > {MAX_GATT_FRAGMENTS}"
            )));
        }
        if fragment_index >= total_fragments {
            return Err(GattError::FragmentationError(
                "Fragment index out of range".to_string(),
            ));
        }
        Ok(Self {
            total_fragments,
            fragment_index,
        })
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> [u8; 4] {
        let mut bytes = [0u8; 4];
        bytes[0..2].copy_from_slice(&self.total_fragments.to_le_bytes());
        bytes[2..4].copy_from_slice(&self.fragment_index.to_le_bytes());
        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, GattError> {
        if bytes.len() < Self::HEADER_SIZE {
            return Err(GattError::ReassemblyError(
                "Fragment header too short".to_string(),
            ));
        }
        let total_fragments = u16::from_le_bytes([bytes[0], bytes[1]]);
        let fragment_index = u16::from_le_bytes([bytes[2], bytes[3]]);
        Self::new(total_fragments, fragment_index)
    }
}

/// GATT Fragmenter for splitting large messages into characteristic-sized writes
pub struct GattFragmenter;

impl GattFragmenter {
    /// Calculate maximum payload size per write
    pub fn max_payload_per_write() -> usize {
        MAX_CHARACTERISTIC_SIZE - GattFragmentHeader::HEADER_SIZE
    }

    /// Split a message into GATT characteristic writes
    pub fn fragment(data: &[u8]) -> Result<Vec<Vec<u8>>, GattError> {
        let max_payload = Self::max_payload_per_write();

        if data.is_empty() {
            return Ok(vec![Vec::new()]);
        }

        let total_fragments = data.len().div_ceil(max_payload);

        if total_fragments > u16::MAX as usize {
            return Err(GattError::FragmentationError(
                "Message too large for GATT fragmentation".to_string(),
            ));
        }

        let mut fragments = Vec::new();

        for (index, chunk) in data.chunks(max_payload).enumerate() {
            let header = GattFragmentHeader::new(total_fragments as u16, index as u16)?;
            let mut fragment = header.to_bytes().to_vec();
            fragment.extend_from_slice(chunk);
            fragments.push(fragment);
        }

        Ok(fragments)
    }
}

/// GATT Reassembler for collecting writes back into complete messages
pub struct GattReassembler;

impl GattReassembler {
    /// Reassemble fragmented messages
    pub fn reassemble(fragments: &[Vec<u8>]) -> Result<Vec<u8>, GattError> {
        if fragments.is_empty() {
            return Ok(Vec::new());
        }

        // Parse first fragment to get total count
        let first_header = GattFragmentHeader::from_bytes(&fragments[0])?;
        let expected_total = first_header.total_fragments as usize;

        if fragments.len() != expected_total {
            return Err(GattError::ReassemblyError(format!(
                "Expected {} fragments, got {}",
                expected_total,
                fragments.len()
            )));
        }

        // Verify all fragments are present and in order
        for (i, fragment) in fragments.iter().enumerate() {
            let header = GattFragmentHeader::from_bytes(fragment)?;
            if header.fragment_index as usize != i {
                return Err(GattError::ReassemblyError(format!(
                    "Fragment out of order: expected index {}, got {}",
                    i, header.fragment_index
                )));
            }
            if header.total_fragments != first_header.total_fragments {
                return Err(GattError::ReassemblyError(
                    "Inconsistent fragment count".to_string(),
                ));
            }
        }

        // Reassemble payload
        let mut result = Vec::new();
        for fragment in fragments {
            if fragment.len() > GattFragmentHeader::HEADER_SIZE {
                result.extend_from_slice(&fragment[GattFragmentHeader::HEADER_SIZE..]);
            }
        }

        Ok(result)
    }
}

/// Write request for the GATT write queue
#[derive(Debug, Clone)]
pub struct GattWriteRequest {
    /// Characteristic to write to
    pub characteristic: GattCharacteristic,
    /// Data to write
    pub data: Vec<u8>,
}

impl GattWriteRequest {
    /// Create a new write request
    pub fn new(characteristic: GattCharacteristic, data: Vec<u8>) -> Result<Self, GattError> {
        if data.len() > MAX_CHARACTERISTIC_SIZE {
            return Err(GattError::FragmentationError(
                "Data exceeds characteristic size".to_string(),
            ));
        }
        Ok(Self {
            characteristic,
            data,
        })
    }
}

/// GATT write queue with backpressure management
pub struct GattWriteQueue {
    queue: VecDeque<GattWriteRequest>,
    max_outstanding: usize,
}

impl GattWriteQueue {
    /// Create a new write queue
    pub fn new(max_outstanding: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            max_outstanding,
        }
    }

    /// Create a write queue with default capacity
    pub fn new_default() -> Self {
        Self::new(DEFAULT_MAX_OUTSTANDING_WRITES)
    }

    /// Check if queue is at capacity
    pub fn is_full(&self) -> bool {
        self.queue.len() >= self.max_outstanding
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get current queue depth
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Enqueue a write request (returns error if queue is full)
    pub fn enqueue(&mut self, request: GattWriteRequest) -> Result<(), GattError> {
        if self.is_full() {
            return Err(GattError::WriteQueueFull);
        }
        self.queue.push_back(request);
        Ok(())
    }

    /// Dequeue the next write request
    pub fn dequeue(&mut self) -> Option<GattWriteRequest> {
        self.queue.pop_front()
    }

    /// Peek at the next request without removing it
    pub fn peek(&self) -> Option<&GattWriteRequest> {
        self.queue.front()
    }

    /// Clear the queue
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gatt_characteristic_uuids() {
        assert_eq!(GattCharacteristic::Write.uuid(), 0xDF02);
        assert_eq!(GattCharacteristic::Notify.uuid(), 0xDF03);
        assert_eq!(GattCharacteristic::Status.uuid(), 0xDF04);
    }

    #[test]
    fn test_gatt_fragment_header_roundtrip() {
        let header = GattFragmentHeader::new(10, 5).expect("Valid header");
        let bytes = header.to_bytes();
        let recovered = GattFragmentHeader::from_bytes(&bytes).expect("Should deserialize");

        assert_eq!(recovered.total_fragments, 10);
        assert_eq!(recovered.fragment_index, 5);
    }

    #[test]
    fn test_gatt_fragment_header_invalid_index() {
        let result = GattFragmentHeader::new(5, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_gatt_fragment_header_zero_total_rejected() {
        let result = GattFragmentHeader::new(0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_gatt_fragment_header_impossible_total_rejected() {
        // A real GattFragmenter can never emit more than MAX_GATT_FRAGMENTS (512)
        // fragments: a 64 KB envelope at 508 payload bytes/fragment is ~130. An
        // unfragmented envelope whose first bytes read as a huge total (e.g. the
        // observed 33025, with a lower index) must therefore not be accepted as a
        // fragment header, else `ble_gatt_ingress` buffers it as a phantom stream.
        assert!(GattFragmentHeader::new(MAX_GATT_FRAGMENTS as u16 + 1, 1).is_err());
        assert!(GattFragmentHeader::new(33025, 17381).is_err());
    }

    #[test]
    fn test_gatt_fragment_header_maximum_total_accepted() {
        // The bound must remain permissive enough for any legitimate stream:
        // exactly MAX_GATT_FRAGMENTS is accepted, with a valid in-range index.
        assert!(
            GattFragmentHeader::new(MAX_GATT_FRAGMENTS as u16, 0).is_ok(),
            "exactly-at-bound total stays valid"
        );
    }

    #[test]
    fn test_gatt_fragment_header_from_bytes_rejects_impossible_total() {
        let mut bytes = vec![0u8; 8];
        bytes[0..2].copy_from_slice(&33025u16.to_le_bytes());
        bytes[2..4].copy_from_slice(&17381u16.to_le_bytes());
        assert!(GattFragmentHeader::from_bytes(&bytes).is_err());
    }

    #[test]
    fn test_gatt_fragmenter_small_message() {
        let data = vec![0u8; 100];
        let fragments = GattFragmenter::fragment(&data).expect("Fragmentation");

        assert_eq!(fragments.len(), 1);
        assert!(fragments[0].len() >= 100);
    }

    #[test]
    fn test_gatt_fragmenter_large_message() {
        let data = vec![0xAAu8; 1000];
        let fragments = GattFragmenter::fragment(&data).expect("Fragmentation");

        assert!(fragments.len() > 1);

        // Each fragment should not exceed MAX_CHARACTERISTIC_SIZE
        for fragment in &fragments {
            assert!(fragment.len() <= MAX_CHARACTERISTIC_SIZE);
        }
    }

    #[test]
    fn test_gatt_fragmenter_empty_message() {
        let data = vec![];
        let fragments = GattFragmenter::fragment(&data).expect("Fragmentation");

        assert_eq!(fragments.len(), 1);
        assert!(fragments[0].is_empty());
    }

    #[test]
    fn test_gatt_reassembler_single_fragment() {
        let header = GattFragmentHeader::new(1, 0).expect("Valid header");
        let mut fragment = header.to_bytes().to_vec();
        fragment.extend_from_slice(b"Hello World");

        let fragments = vec![fragment];
        let result = GattReassembler::reassemble(&fragments).expect("Reassembly");

        assert_eq!(result, b"Hello World");
    }

    #[test]
    fn test_gatt_fragmenter_reassembler_roundtrip() {
        let original = vec![0xBBu8; 2000];
        let fragments = GattFragmenter::fragment(&original).expect("Fragmentation");
        let reassembled = GattReassembler::reassemble(&fragments).expect("Reassembly");

        assert_eq!(reassembled, original);
    }

    #[test]
    fn test_gatt_reassembler_wrong_fragment_count() {
        let header = GattFragmentHeader::new(3, 0).expect("Valid header");
        let fragment = header.to_bytes().to_vec();

        let fragments = vec![fragment];
        let result = GattReassembler::reassemble(&fragments);

        assert!(result.is_err());
    }

    #[test]
    fn test_gatt_reassembler_out_of_order() {
        let header1 = GattFragmentHeader::new(2, 1).expect("Valid header");
        let header2 = GattFragmentHeader::new(2, 0).expect("Valid header");

        let mut fragment1 = header1.to_bytes().to_vec();
        fragment1.extend_from_slice(b"World");

        let mut fragment2 = header2.to_bytes().to_vec();
        fragment2.extend_from_slice(b"Hello");

        let fragments = vec![fragment1, fragment2];
        let result = GattReassembler::reassemble(&fragments);

        assert!(result.is_err());
    }

    #[test]
    fn test_gatt_write_request_creation() {
        let data = vec![0u8; 100];
        let request =
            GattWriteRequest::new(GattCharacteristic::Write, data).expect("Valid request");

        assert_eq!(request.characteristic, GattCharacteristic::Write);
        assert_eq!(request.data.len(), 100);
    }

    #[test]
    fn test_gatt_write_request_max_size() {
        let data = vec![0u8; MAX_CHARACTERISTIC_SIZE + 1];
        let result = GattWriteRequest::new(GattCharacteristic::Write, data);

        assert!(result.is_err());
    }

    #[test]
    fn test_gatt_write_queue_empty() {
        let queue = GattWriteQueue::new_default();

        assert!(queue.is_empty());
        assert!(!queue.is_full());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_gatt_write_queue_enqueue_dequeue() {
        let mut queue = GattWriteQueue::new(10);

        let request = GattWriteRequest::new(GattCharacteristic::Write, vec![0x42; 50])
            .expect("Valid request");

        queue.enqueue(request).expect("Enqueue success");
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());

        let dequeued = queue.dequeue().expect("Dequeue success");
        assert_eq!(dequeued.data.len(), 50);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_gatt_write_queue_backpressure() {
        let mut queue = GattWriteQueue::new(2);

        let request1 = GattWriteRequest::new(GattCharacteristic::Write, vec![0x42; 50])
            .expect("Valid request");
        queue.enqueue(request1).expect("First enqueue");

        let request2 = GattWriteRequest::new(GattCharacteristic::Write, vec![0x43; 50])
            .expect("Valid request");
        queue.enqueue(request2).expect("Second enqueue");

        assert!(queue.is_full());

        let request3 = GattWriteRequest::new(GattCharacteristic::Write, vec![0x44; 50])
            .expect("Valid request");
        let result = queue.enqueue(request3);

        assert!(result.is_err());
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_gatt_write_queue_peek() {
        let mut queue = GattWriteQueue::new(10);

        let request = GattWriteRequest::new(GattCharacteristic::Notify, vec![0x45; 30])
            .expect("Valid request");
        queue.enqueue(request).expect("Enqueue success");

        let peeked = queue.peek().expect("Peek success");
        assert_eq!(peeked.characteristic, GattCharacteristic::Notify);
        assert_eq!(queue.len(), 1); // Still in queue

        queue.dequeue();
        assert!(queue.peek().is_none());
    }

    #[test]
    fn test_gatt_write_queue_clear() {
        let mut queue = GattWriteQueue::new(10);

        for _ in 0..5 {
            let request = GattWriteRequest::new(GattCharacteristic::Write, vec![0x42; 20])
                .expect("Valid request");
            queue.enqueue(request).expect("Enqueue success");
        }

        assert_eq!(queue.len(), 5);
        queue.clear();
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_gatt_fragment_payload_size() {
        let max_payload = GattFragmenter::max_payload_per_write();
        assert_eq!(max_payload, MAX_CHARACTERISTIC_SIZE - 4); // 4-byte header
    }

    #[test]
    fn test_gatt_characteristic_all_variants() {
        let characteristics = vec![
            GattCharacteristic::Write,
            GattCharacteristic::Notify,
            GattCharacteristic::Status,
        ];

        for char in characteristics {
            assert!(char.uuid() > 0);
        }
    }

    #[test]
    fn test_gatt_write_queue_fifo_order() {
        let mut queue = GattWriteQueue::new(10);

        let request1 = GattWriteRequest::new(GattCharacteristic::Write, vec![0x01]).expect("Valid");
        let request2 = GattWriteRequest::new(GattCharacteristic::Write, vec![0x02]).expect("Valid");
        let request3 = GattWriteRequest::new(GattCharacteristic::Write, vec![0x03]).expect("Valid");

        queue.enqueue(request1).expect("Enqueue 1");
        queue.enqueue(request2).expect("Enqueue 2");
        queue.enqueue(request3).expect("Enqueue 3");

        assert_eq!(queue.dequeue().expect("Dequeue").data[0], 0x01);
        assert_eq!(queue.dequeue().expect("Dequeue").data[0], 0x02);
        assert_eq!(queue.dequeue().expect("Dequeue").data[0], 0x03);
    }
}
