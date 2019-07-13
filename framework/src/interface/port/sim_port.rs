use super::super::{PacketRx, PacketTx};
use super::PortStats;
use allocators::*;
use common::*;
use native::mbuf::{MBuf, MAX_MBUF_SIZE};
use native::{mbuf_alloc_bulk, mbuf_free_bulk};
use std::fmt;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub struct SimulatePort {
    stats_rx: Arc<CacheAligned<PortStats>>,
    stats_tx: Arc<CacheAligned<PortStats>>,
}

#[derive(Clone)]
pub struct SimulateQueue {
    stats_rx: Arc<CacheAligned<PortStats>>,
    stats_tx: Arc<CacheAligned<PortStats>>,
}

impl fmt::Display for SimulateQueue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Simulate queue")
    }
}

impl PacketTx for SimulateQueue {
    #[inline]
    fn send(&self, pkts: &mut [*mut MBuf]) -> Result<u32> {
        let len = pkts.len() as i32;
        let update = self.stats_tx.stats.load(Ordering::Relaxed) + len as usize;
        self.stats_tx.stats.store(update, Ordering::Relaxed);
        mbuf_free_bulk(pkts.as_mut_ptr(), len);
        Ok(len as u32)
    }
}

impl PacketRx for SimulateQueue {
    /// Send a batch of packets out this PortQueue. Note this method is internal to NetBricks (should not be directly
    /// called).
    #[inline]
    fn recv(&self, pkts: &mut [*mut MBuf]) -> Result<u32> {
        let len = pkts.len() as i32;
        let status = mbuf_alloc_bulk(pkts.as_mut_ptr(), MAX_MBUF_SIZE, len);
        let alloced = if status == 0 { len } else { 0 };
        let update = self.stats_rx.stats.load(Ordering::Relaxed) + alloced as usize;
        self.stats_rx.stats.store(update, Ordering::Relaxed);
        Ok(alloced as u32)
    }
}

impl SimulatePort {
    pub fn new(_queues: i32) -> Result<Arc<SimulatePort>> {
        Ok(Arc::new(SimulatePort {
            stats_rx: Arc::new(PortStats::new()),
            stats_tx: Arc::new(PortStats::new()),
        }))
    }

    pub fn new_simulate_queue(&self, _queue: i32) -> Result<CacheAligned<SimulateQueue>> {
        Ok(CacheAligned::allocate(SimulateQueue {
            stats_rx: self.stats_rx.clone(),
            stats_tx: self.stats_tx.clone(),
        }))
    }

    /// Get stats for an RX/TX queue pair.
    pub fn stats(&self) -> (usize, usize) {
        (
            self.stats_rx.stats.load(Ordering::Relaxed),
            self.stats_tx.stats.load(Ordering::Relaxed),
        )
    }
}
