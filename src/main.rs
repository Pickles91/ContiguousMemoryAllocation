use std::{fs, ops::Deref};

mod log;

use contiguous_memory_allocation::{
    parse_config,
    strategies::{BestFit, MemAllocator, MemoryRegion, MemoryRequest, NextFit, Pid, WorstFit},
};
use rand::{thread_rng, Rng};

use crate::log::draw_gui;

fn main() {
    // get the config file as the first argument to the process.
    let mut args = std::env::args();
    // first arg is always executing process name
    let _proc_name = args.next().unwrap();

    let file = std::path::PathBuf::from(
        args.next()
            .expect("Please pass in the filepath to the config"),
    );

    let config = parse_config(fs::read_to_string(file).unwrap().as_str()).unwrap();
    println!("Loaded config: {:#?}", config);
    let worst = WorstFit::new(config.memory_max);
    let best = BestFit::new(config.memory_max);
    let next = NextFit::new(config.memory_max);
    let requests = gen_processes(config.num_proc, config.proc_size_max, config.max_proc_time);
    let results = std::sync::Mutex::new([None, None, None]);
    // we do this threaded bc I accidentally did a sleep, and I thought my simulation was just kind of slow...
    // turns out no, it's actually fast - but I ended up having threaded it anyways to do it concurrently so
    // here you go.
    std::thread::scope(|s| {
        s.spawn(|| results.lock().unwrap()[0] = Some(driver(next, &requests)));
        s.spawn(|| results.lock().unwrap()[1] = Some(driver(best, &requests)));
        s.spawn(|| results.lock().unwrap()[2] = Some(driver(worst, &requests)));
    });
    draw_gui(
        results
            .into_inner()
            .unwrap()
            .map(|i| i.unwrap())
            .try_into()
            .unwrap(),
        config,
    );
}

fn driver<T: MemAllocator>(mut alloc: T, requests: &[MemoryRequest]) -> Vec<Vec<MemoryRegion>> {
    for req in requests {
        alloc = alloc.request(*req);
    }
    let mut out: Vec<Vec<MemoryRegion>> = vec![];
    loop {
        let (mem, alloc_new) = alloc.tick();
        alloc = alloc_new;
        if mem.len() == 2 {
            break;
        }
        out.push(mem);
    }
    return out;
}

fn gen_processes(num_processes: u32, max_size: u32, lifetime: u32) -> Vec<MemoryRequest> {
    let mut rng = thread_rng();
    (0..num_processes)
        .map(|i| MemoryRequest {
            process: Pid(i),
            size: rng.gen_range(0..max_size),
            lifetime: rng.gen_range(0..lifetime / 1000),
        })
        .collect()
}
