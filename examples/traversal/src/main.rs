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
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::BuildHasherDefault;
use std::hash::{Hash, Hasher};
use std::io::stdout;
use std::io::Write;
use std::sync::Arc;

type FnvHash = BuildHasherDefault<FnvHasher>;

thread_local! {
    // Per flow packet counter
    pub static FLOW_MAP: RefCell<HashMap<Flow, u64, FnvHash>> = {
        let m = HashMap::with_hasher(Default::default());
        RefCell::new(m)
    };

    // Per flow packet payload hash
    pub static FLOW_PAYLOAD_MAP: RefCell<HashMap<Flow, Vec<u64>, FnvHash>> = {
        let m = HashMap::with_hasher(Default::default());
        RefCell::new(m)
    };
}

pub fn calculate_hash<T: Hash>(t: T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub fn hash_it(a: &[u8]) -> u64 {
    calculate_hash(a)
}

fn install<T, S>(ports: Vec<T>, sched: &mut S)
where
    T: PacketRx + PacketTx + Display + Clone + 'static,
    S: Scheduler + Sized,
{
    println!("Receiving started");
    let pipelines: Vec<_> = ports
        .iter()
        .map(move |port| {
            // println!("Port: {:?}", port);
            ReceiveBatch::new(port.clone())
                .map(|p| traversal(p))
                .sendall(port.clone())
        })
        .collect();

    println!("Running {} pipelines", pipelines.len()+1);
    for pipeline in pipelines {
        // println!("Pipeline: {:?}", pipeline);
        sched.add_task(pipeline).unwrap();
    }
}

fn traversal(packet: RawPacket) -> Result<Tcp<Ipv4>> {
    let mut ethernet = packet.parse::<Ethernet>()?;
    ethernet.swap_addresses();
    let v4 = ethernet.parse::<Ipv4>()?;
    let tcp = v4.parse::<Tcp<Ipv4>>()?;
    let flow = tcp.flow();
    println!("{}", flow);

    println!("before flow_map");
    stdout().flush().unwrap();

    let payload = tcp.get_payload();
    let hash = hash_it(payload);

    // no integrity check
    // FLOW_MAP.with(|flow_map| {
    //     packet.payload = flow_map
    //     println!("inside flow_map");
    //     stdout().flush().unwrap();
    //     println!("{}", flow);
    //     stdout().flush().unwrap();
    //     *((*flow_map.borrow_mut()).entry(flow).or_insert(0)) += 1;
    // });

    // integrity check
    FLOW_PAYLOAD_MAP.with(|flow_map| {
        println!("inside flow_map");
        stdout().flush().unwrap();
        println!("Flow: {}, Hash: {}", flow, hash);
        stdout().flush().unwrap();
        (*flow_map.borrow_mut())
            .entry(flow)
            .or_insert(vec![hash])
            .push(hash);
    });

    Ok(tcp)
}

fn main() -> Result<()> {
    let configuration = load_config()?;
    println!("{}", configuration);
    let mut context = initialize_system(&configuration)?;
    println!("PKT NUM: {}", PKT_NUM);
    context.run(Arc::new(install), PKT_NUM); // will trap in the run() and return after finish
    Ok(())
}
