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

fn prev_traversal(packet: RawPacket) -> Result<Tcp<Ipv4>> {
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

pub fn traversal<T: 'static + Batch<Header = NullHeader>>(
    parent: T,
) -> TransformBatch<MacHeader, ParsedBatch<MacHeader, T>> {
    parent.parse::<MacHeader>().transform(box move |pkt| {
        assert!(pkt.refcnt() == 1);
        let hdr = pkt.get_mut_header();
        hdr.swap_addresses();
    })
}
