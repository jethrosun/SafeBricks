/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate lazy_static;
extern crate mylib;
extern crate sharedring;
extern crate tokio;
#[macro_use]
extern crate clap;

#[link(name = "mapping", kind = "static")]
extern "C" {
    fn mapping();
}

use mylib::config::{get_duration, load_config, NetBricksConfiguration, NUM_RXD, NUM_TXD};
use mylib::haproxy::{parse_args, run_client, run_server};
use sharedring::ring_buffer::*;

use std::fmt::Display;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::process;
use std::sync::atomic::{compiler_fence, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use tokio::timer::{Delay, Interval};
type TokioRuntime = tokio::runtime::current_thread::Runtime;

// const PKT_NUM: u64 = (8 * 1024 * 1024);
// const PRINT_INTER: u64 = (1024 * 1024);

// pull_count;
lazy_static! {
    static ref BATCH_CNT: Mutex<Vec<u64>> = {
        let batch_cnt = (0..1).map(|_| 0 as u64).collect();
        Mutex::new(batch_cnt)
    };
}

// pull_count;
lazy_static! {
    static ref BATCH_CNT_SGX: Mutex<Vec<u64>> = {
        let batch_cnt = (0..1).map(|_| 0 as u64).collect();
        Mutex::new(batch_cnt)
    };
}

// TODO: extract the config and ring_buffer from the pkupuller (separated from dpdk logic)
// Reason: system will crash if dpdk crate co-exists with second enclave creation.
fn main() {
    unsafe {
        mapping();
    };

    let configuration = load_config().unwrap();
    println!("{}", configuration);

    let port_num = configuration.ports[0].rx_queues.len();
    println!("ports number: {}", port_num);

    // The regular way to get core ids are not going work as we have configured isol cpus to reduce context switches for DPDK and our things.
    // We want to cause equal pressure to all of the cores for CPU contention
    let mut core_ids = Vec::new();
    for idx in 0..6 {
        // cores [CoreId { id: 0 }, CoreId { id: 1 }]
        core_ids.push(core_affinity::CoreId { id: idx });
    }
    // let core_ids = core_affinity::get_core_ids().unwrap();
    println!(
        "core_affinity detect: # available cores: {}",
        core_ids.len()
    );
    assert!(
        core_ids.len() >= port_num + 1,
        "# available cores is not enough"
    );
    // one core for pktpuller, one core for normal linux monitoring.

    let mut server_count: u64 = 0;
    let mut client_count: u64 = 0;
    let file = parse_args().unwrap();

    let mut recvq_ring: Vec<RingBuffer> = Vec::new();
    let mut sendq_ring: Vec<RingBuffer> = Vec::new();

    for i in 0..port_num {
        // Create two shared queue: recvq and sendq;
        recvq_ring.push(
            unsafe {
                RingBuffer::new_in_heap(
                    (NUM_RXD) as usize,
                    &format!("{}_{}", RECVQ_PREFIX, i),
                    false,
                )
            }
            .unwrap(),
        );
        sendq_ring.push(
            unsafe {
                RingBuffer::new_in_heap(
                    (NUM_TXD) as usize,
                    &format!("{}_{}", SENDQ_PREFIX, i),
                    false,
                )
            }
            .unwrap(),
        );

        let core_ids_sgx = core_ids[i + 1].clone();
        let file_core = file.clone();
        let server = thread::spawn(move || {
            core_affinity::set_for_current(core_ids_sgx);
            run_server(file_core).unwrap();
            // server_count += run_server_thread().unwrap();
        });

        let recvq_addr_u64: u64 = recvq_ring[i].head.my_usize as u64; // *mut usize
        let sendq_addr_u64: u64 = sendq_ring[i].head.my_usize as u64;

        println!(
            "recvq_addr {}, sendq_addr {}",
            recvq_addr_u64, sendq_addr_u64
        );
        // send recvq_addr and sendq_addr to the enclave through TCP tunnel.
        run_client(recvq_addr_u64, sendq_addr_u64).unwrap(); // recvq_addr, sendq_addr

        println!(
            "  recvq: head {} vs. tail {}",
            recvq_ring[i].head(),
            recvq_ring[i].tail()
        );
        println!(
            "  sendq: head {} vs. tail {}",
            sendq_ring[i].head(),
            sendq_ring[i].tail()
        );
    }

    let recvq_ring_r = recvq_ring.clone();
    ctrlc::set_handler(move || {
        for i in 0..port_num {
            compiler_fence(Ordering::Release);
            recvq_ring_r[i as usize].set_size(STOP_MARK as usize);
        }
        thread::sleep(std::time::Duration::from_secs(1)); // wait until server in enclave sets up;
        process::exit(1);
    })
    .expect("Error setting Ctrl-C handler");

    let mut tokio_rt = TokioRuntime::new().unwrap();
    let when = Instant::now() + Duration::from_secs(get_duration());
    let main_loop = Delay::new(when);
    let res = tokio_rt.block_on(main_loop);
    // seems like if sgx-runner process exits, the sgx threads will also exit: at least htop will not show them.
    // maybe it is because of sgx enclave property.

    // loop{
    //     for i in 0..port_num {
    //         println!("waiting for the ctrl+c");
    //         println!("  recvq: head {} vs. tail {}", recvq_ring[i].head(), recvq_ring[i].tail());
    //         println!("  sendq: head {} vs. tail {}", sendq_ring[i].head(), sendq_ring[i].tail());
    //         thread::sleep(std::time::Duration::from_secs(1));
    //     }
    // }
    // directly exit and let enclaves run.
    // No, you cannot exit, since the ringbuffer will dropped if you leave.
    // have fixed it in ringbuffer using shm_master bool value.
}
