use super::super::{PacketRx, PacketTx};
use super::PortStats;
use allocators::*;
use common::*;
use native::mbuf::{MBuf, MAX_MBUF_SIZE};
use native::{mbuf_alloc_bulk, mbuf_free_bulk};
use std::fmt;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use config::{PortConfiguration, NUM_RXD, NUM_TXD};
use operators::BATCH_SIZE;

use std::io::stdout;
use std::io::Write;

use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::thread;
use std::sync::Mutex;
use std::slice;

use heap_ring::ring_buffer::*;

// pkt_count;
lazy_static!{
    static ref BATCH_CNT: Mutex<Vec<u64>> = {
        let batch_cnt = (0..1).map(|_| 0 as u64).collect();        
        Mutex::new(batch_cnt)
    };
}

pub struct SimulatePort {
    stats_rx: Arc<CacheAligned<PortStats>>,
    stats_tx: Arc<CacheAligned<PortStats>>,
}

impl fmt::Debug for SimulatePort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Simulate port")
    }
}

#[derive(Clone)]
pub struct SimulateQueue {
    stats_rx: Arc<CacheAligned<PortStats>>,
    stats_tx: Arc<CacheAligned<PortStats>>,
    recvq_ring: RingBuffer,
    sendq_ring: RingBuffer,
}

impl fmt::Display for SimulateQueue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Simulate queue")
    }
}

impl PacketTx for SimulateQueue {
    #[inline]
    fn send(&self, pkts: &mut [*mut MBuf]) -> Result<u32> {
        let mut mbufs = pkts.to_vec();
        let len = mbufs.len() as i32;
        // let len = pkts.len() as i32;
        let update = self.stats_tx.stats.load(Ordering::Relaxed) + len as usize;
        self.stats_tx.stats.store(update, Ordering::Relaxed);

        // push len mbuf pointers to sendq.
        if !mbufs.is_empty() {
            let mut to_send = mbufs.len();
            while to_send > 0 {
                let b_u8_p = unsafe{ (&(*(mbufs[0])) as *const MBuf) as *const u8 };
                let b_u8_array = unsafe{ slice::from_raw_parts(b_u8_p, to_send * 8) };
                let sent = self.sendq_ring.write_at_tail(b_u8_array) / 8;
                println!("{}, {}", sent, self.sendq_ring.tail());
                // thread::sleep(std::time::Duration::from_secs(1));// for debugging;
            
                to_send -= sent;
                if to_send > 0 {
                    mbufs.drain(..sent);
                }
            }
            unsafe {
                unsafe{ mbufs.set_len(0) };
            }
        }
        // mbuf_free_bulk(pkts.as_mut_ptr(), len);
        Ok(len as u32)
    }
}

impl PacketRx for SimulateQueue {
    /// Send a batch of packets out this PortQueue. Note this method is internal to NetBricks (should not be directly
    /// called).
    #[inline]
    fn recv(&self, pkts: &mut [*mut MBuf]) -> Result<u32> {
        let mut mbufs = pkts.to_vec();
        let len = mbufs.len() as i32;
        // let len = pkts.len() as i32;

        // println!("recv0"); stdout().flush().unwrap();
        
        // pull packet from recvq;
        let b_u8_p_mut = unsafe{ (&mut (*(mbufs[0])) as *mut MBuf) as *mut u8 };
        let b_u8_array_mut = unsafe{ slice::from_raw_parts_mut(b_u8_p_mut, BATCH_SIZE * 8) };
        let recv_pkt_num_from_enclave = self.recvq_ring.read_from_head(b_u8_array_mut) / 8;
        unsafe{ mbufs.set_len(recv_pkt_num_from_enclave) }; 
        
        BATCH_CNT.lock().unwrap()[0] += 1;
        let batch_cnt = BATCH_CNT.lock().unwrap()[0];
        if batch_cnt % (1024 * 1024) == 0 || recv_pkt_num_from_enclave != 0{
            println!("{}, {}, {}", recv_pkt_num_from_enclave, self.recvq_ring.tail(), self.recvq_ring.head());
        }

        // let status = mbuf_alloc_bulk(pkts.as_mut_ptr(), MAX_MBUF_SIZE, len);
        // println!("recv1 {}", status); stdout().flush().unwrap();
        // let alloced = if status == 0 { len } else { 0 };
        let alloced = recv_pkt_num_from_enclave;
        let update = self.stats_rx.stats.load(Ordering::Relaxed) + alloced as usize;
        self.stats_rx.stats.store(update, Ordering::Relaxed);
        // println!("rx {}, tx {}", self.stats_rx.stats.load(Ordering::Relaxed), self.stats_tx.stats.load(Ordering::Relaxed)); stdout().flush().unwrap();
        Ok(alloced as u32)
    }
}

fn fib(n: u64) -> u64{
    if n == 0{
        return 0;
    }
    else if n == 1{
        return 1;
    }
    else{
        return fib(n - 1) + fib(n - 2); 
    }
}

impl SimulatePort {
    pub fn new(port_config: &PortConfiguration) -> Result<Arc<SimulatePort>> {        
        Ok(Arc::new(SimulatePort {
            stats_rx: Arc::new(PortStats::new()),
            stats_tx: Arc::new(PortStats::new()),
        }))
    }

    pub fn new_simulate_queue(&self, _queue: i32) -> Result<CacheAligned<SimulateQueue>> {
        let listener = TcpListener::bind("localhost:6010")?;
        let (stream, peer_addr) = listener.accept()?;
        let peer_addr = peer_addr.to_string();
        let local_addr = stream.local_addr()?;
        eprintln!(
            "App:: accept  - local address is {}, peer address is {}",
            local_addr, peer_addr
        );

        let mut reader = BufReader::new(stream);
        let mut message = String::new();
        
        let read_bytes = reader.read_line(&mut message)?;
        print!("{}", message);
        let queue_addr: Vec<u64> = 
                message.trim().split(' ')
            .map(|s| s.parse().unwrap())
            .collect();
        println!("{:?}", queue_addr);
            // fib(30);

        Ok(CacheAligned::allocate(SimulateQueue {
            stats_rx: self.stats_rx.clone(),
            stats_tx: self.stats_tx.clone(),
            recvq_ring: unsafe{ RingBuffer::attach_in_heap((NUM_RXD * 8) as usize, queue_addr[0]).unwrap() }, 
            sendq_ring: unsafe{ RingBuffer::attach_in_heap((NUM_TXD * 8) as usize, queue_addr[1]).unwrap() },
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
