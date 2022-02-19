extern crate fnv;
#[macro_use]
extern crate lazy_static;
extern crate netbricks;

use self::nf::*;
use fnv::FnvHasher;
use netbricks::common::Result;
use netbricks::config::load_config;
use netbricks::interface::{PacketRx, PacketTx};
use netbricks::operators::{Batch, ReceiveBatch};
use netbricks::packets::ip::v4::Ipv4;
use netbricks::packets::ip::Flow;
use netbricks::packets::{Ethernet, Packet, RawPacket, Tcp};
use netbricks::scheduler::Scheduler;
use netbricks::scheduler::{initialize_system, PKT_NUM};
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::BuildHasherDefault;
use std::hash::{Hash, Hasher};
use std::io::stdout;
use std::io::Write;
use std::sync::Arc;

mod nf;

type FnvHash = BuildHasherDefault<FnvHasher>;

fn install<T, S>(ports: Vec<T>, sched: &mut S)
where
    T: PacketRx + PacketTx + Display + Clone + 'static,
    S: Scheduler + Sized,
{
    println!("Receiving started");\
    let pipelines: Vec<_> = ports
        .iter()
        .map(|port| traversal(ReceiveBatch::new(port.clone())).send(port.clone()))
        .collect();

    println!("Running {} pipelines", pipelines.len()+1);
    for pipeline in pipelines {
        // println!("Pipeline: {:?}", pipeline);
        sched.add_task(pipeline).unwrap();
    }
}

fn main() -> Result<()> {
    let configuration = load_config()?;
    println!("{}", configuration);
    let mut context = initialize_system(&configuration)?;
    println!("PKT NUM: {}", PKT_NUM);
    context.run(Arc::new(install), PKT_NUM); // will trap in the run() and return after finish
    Ok(())
}
