use std::{io::Stdout, time::Duration};

use rand::{seq::SliceRandom, thread_rng, Rng};

use contiguous_memory_allocation::{
    strategies::{MemoryRegion, Pid},
    Config,
};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::CrosstermBackend,
    layout::{self, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{
        canvas::{Canvas, Label, Rectangle},
        Bar, BarChart, BarGroup, Paragraph,
    },
    Terminal,
};

struct Gui {
    // a list of different memories the program has had over it's
    // lifetime. You can get the Nth state of RAM by indexing to mem[n]
    mem: [Vec<Vec<MemoryRegion>>; 3],
}

#[derive(PartialEq, Eq)]
enum ProcessOrFree {
    Process(Pid, i32),
    Free,
}

impl Gui {
    fn new(mem: [Vec<Vec<MemoryRegion>>; 3]) -> Self {
        Self { mem }
    }
    fn frames(mem: &[MemoryRegion]) -> Vec<(ProcessOrFree, u32)> {
        mem.windows(2)
            .map(|item| {
                let [region, next_region]: [MemoryRegion; 2] = item.try_into().unwrap();
                if let Some((pid, lifetime)) = region.0 {
                    (
                        ProcessOrFree::Process(pid, lifetime),
                        next_region.1 - region.1,
                    )
                } else {
                    (ProcessOrFree::Free, next_region.1 - region.1)
                }
            })
            .collect::<Vec<_>>()
    }
    fn stats(info: &[(ProcessOrFree, u32)]) -> String {
        let total_free: u32 = info
            .iter()
            .filter_map(|(process_or_free, size)| {
                if *process_or_free == ProcessOrFree::Free {
                    Some(size)
                } else {
                    None
                }
            })
            .sum();
        let num_holes: u32 = info
            .iter()
            .map(|(process_or_free, _)| {
                if *process_or_free == ProcessOrFree::Free {
                    1
                } else {
                    0
                }
            })
            .sum();
        let total_full: u32 = info.iter().map(|(_, size)| size).sum();
        let percentage = total_free * 100 / total_full;
        format!("total free: {total_free}, percentage free: {percentage}, holes: {num_holes}")
    }
    fn draw_ram(info: &[(ProcessOrFree, u32)]) -> String {
        let mut out = String::new();
        for (proc_or_free, size) in info {
            match proc_or_free {
                ProcessOrFree::Process(pid, lifetime) => {
                    out += &format!("p{pid}[{lifetime}s]({size}KB)|", pid = pid.0)
                }
                ProcessOrFree::Free => out += &format!("FREE({size}KB)|"),
            }
        }
        out
    }
    fn draw_gui(&mut self, config: Config) {
        println!("do you want auto mode? y/n");
        let mut buff = String::new();
        std::io::stdin().read_line(&mut buff).unwrap();
        if buff.trim().to_lowercase() == "y" {
            println!("screen will update every 2 seconds");
        }
        for i in 0..(*[self.mem[0].len(), self.mem[1].len(), self.mem[2].len()]
            .iter()
            .min()
            .unwrap())
        {
            println!("------------------------------------------------------");
            println!("best");
            println!("{}", Self::stats(&Self::frames(&self.mem[1][i])));
            println!("[{}]", Self::draw_ram(&Self::frames(&self.mem[1][i])));
            println!();
            println!("next");
            println!("{}", Self::stats(&Self::frames(&self.mem[0][i])));
            println!("[{}]", Self::draw_ram(&Self::frames(&self.mem[0][i])));
            println!();
            println!("worst");
            println!("{}", Self::stats(&Self::frames(&self.mem[2][i])));
            println!("[{}]", Self::draw_ram(&Self::frames(&self.mem[2][i])));
            if buff.trim().to_lowercase() == "y" {
                std::thread::sleep(Duration::from_secs(2));
            } else {
                println!("press enter to advance");
                buff.clear();
                std::io::stdin().read_line(&mut buff).unwrap();
            }
        }
    }
}

pub(crate) fn draw_gui(mem: [Vec<Vec<MemoryRegion>>; 3], config: Config) {
    Gui::new(mem).draw_gui(config);
}
