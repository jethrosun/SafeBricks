extern crate fnv;
#[macro_use]
extern crate lazy_static;
extern crate netbricks;
use fnv::FnvHasher;
use netbricks::common::Result;
use netbricks::config::load_config;
use netbricks::interface::{PacketRx, PacketTx};
use netbricks::operators::{mpsc_batch, Batch, Enqueue, MpscProducer, ReceiveBatch};
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

type FnvHash = BuildHasherDefault<FnvHasher>;

fn install<T, S>(ports: Vec<T>, sched: &mut S)
where
    T: PacketRx + PacketTx + Display + Clone + 'static,
    S: Scheduler + Sized,
{
    println!("Forwarding pipeline");

    let (producer, outbound) = mpsc_batch();
    let outbound = outbound.send(ports[0].clone());
    sched.add_task(outbound).unwrap();

    println!("Receiving started");

    let pipelines: Vec<_> = ports
        .iter()
        .map(move |port| {
            let producer = producer.clone();
            ReceiveBatch::new(port.clone())
                .map(move |p| forward(p, &producer))
                .filter(|_| false)
                .send(port.clone())
        })
        .collect();

    println!("Running {} pipelines", pipelines.len() + 1);
    for pipeline in pipelines {
        sched.add_task(pipeline).unwrap();
    }
}

fn forward(packet: RawPacket, producer: &MpscProducer) -> Result<RawPacket> {
    producer.enqueue(packet);

    let bogus = RawPacket::new()?;
    Ok(bogus)
}

fn main() -> Result<()> {
    let configuration = load_config()?;
    println!("{}", configuration);
    let mut context = initialize_system(&configuration)?;
    println!("PKT NUM: {}", PKT_NUM);
    context.run(Arc::new(install), PKT_NUM); // will trap in the run() and return after finish
    Ok(())
}
