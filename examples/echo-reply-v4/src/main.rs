extern crate fnv;
#[macro_use]
extern crate lazy_static;
extern crate netbricks;
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
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::BuildHasherDefault;
use std::io::stdout;
use std::io::Write;
use std::sync::Arc;

type FnvHash = BuildHasherDefault<FnvHasher>;

thread_local! {
    pub static FLOW_MAP: RefCell<HashMap<Flow, u64, FnvHash>> = {
        let m = HashMap::with_hasher(Default::default());
        RefCell::new(m)
    };
}

fn install<T, S>(ports: Vec<T>, sched: &mut S)
where
    T: PacketRx + PacketTx + Display + Clone + 'static,
    S: Scheduler + Sized,
{
    println!("Echo reply pipeline");

    let (producer, outbound) = mpsc_batch();
    let outbound = outbound.send(ports[0].clone());
    sched.add_task(outbound).unwrap();

    println!("Receiving started");

    let pipelines: Vec<_> = ports
        .iter()
        .map(move |port| {
            let producer = producer.clone();
            ReceiveBatch::new(port.clone())
                .map(move |p| reply_echo(p, &producer))
                .filter(|_| false)
                .send(port.clone())
        })
        .collect();

    println!("Running {} pipelines", pipelines.len());
    for pipeline in pipelines {
        sched.add_task(pipeline).unwrap();
    }
}

fn reply_echo(packet: RawPacket, producer: &MpscProducer) -> Result<Icmpv4<Ipv4, EchoRequest>> {
    let reply = RawPacket::new()?;

    let ethernet = packet.parse::<Ethernet>()?;
    let mut reply = reply.push::<Ethernet>()?;
    reply.set_src(ethernet.dst());
    reply.set_dst(ethernet.src());
    reply.set_ether_type(EtherTypes::Ipv4);

    let ipv4 = ethernet.parse::<Ipv4>()?;
    let mut reply = reply.push::<Ipv4>()?;
    reply.set_src(ipv4.dst());
    reply.set_dst(ipv4.src());
    reply.set_next_header(ProtocolNumbers::Icmpv4);

    let icmpv4 = ipv4.parse::<Icmpv4<Ipv4, ()>>()?;
    let echo = icmpv4.downcast::<EchoRequest>()?;
    let mut reply = reply.push::<Icmpv4<Ipv4, EchoReply>>()?;
    reply.set_identifier(echo.identifier());
    reply.set_seq_no(echo.seq_no());
    reply.set_data(echo.data())?;
    reply.cascade();

    producer.enqueue(reply.reset());

    Ok(echo)
}

fn main() -> Result<()> {
    let configuration = load_config()?;
    println!("{}", configuration);
    let mut context = initialize_system(&configuration)?;
    println!("PKT NUM: {}", PKT_NUM);
    context.run(Arc::new(install), PKT_NUM); // will trap in the run() and return after finish
    Ok(())
}
