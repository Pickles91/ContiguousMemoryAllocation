use std::{io::Stdout, ops::Add, time::Duration};

use rand::{seq::SliceRandom, thread_rng, Rng};

use contiguous_memory_allocation::{
    strategies::{MemoryRegion, MemoryRequest, Pid},
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
    frame_info: [Vec<(Vec<MemoryRegion>, Vec<MemoryRequest>)>; 3],
}

#[derive(PartialEq, Eq)]
enum ProcessOrFree {
    Process(Pid, i32),
    Free,
}

impl Gui {
    fn new(frame_info: [Vec<(Vec<MemoryRegion>, Vec<MemoryRequest>)>; 3]) -> Self {
        Self { frame_info }
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
    fn stats(info: &[(ProcessOrFree, u32)], requests: &[MemoryRequest]) -> String {
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
        format!(
            "Total Free: {total_free}, Percentage Free: {percentage}, Hole(s): {num_holes}\nREMAINING REQUESTS: [{}]",
            requests
                .iter()
                .map(|req| {
                    format!(
                        "P{pid}[{lifetime}s]({size}KB)",
                        pid = req.process.0,
                        lifetime = req.lifetime,
                        size = req.size
                    )
                })
                .collect::<Vec<_>>()
                .join("|")
        )
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
        println!("Do you want auto mode? y/n");
        let mut buff = String::new();
        std::io::stdin().read_line(&mut buff).unwrap();
        if buff.trim().to_lowercase() == "y" {
            println!("Screen will update every 2 seconds.");
        }
        for i in 0..(*[
            self.frame_info[0].len(),
            self.frame_info[1].len(),
            self.frame_info[2].len(),
        ]
        .iter()
        .min()
        .unwrap())
        {
            println!("------------------------------------------------------");
            println!("Best Fit:");
            println!(
                "[{}]",
                Self::draw_ram(&Self::frames(&self.frame_info[1][i].0))
            );
            println!(
                "{}",
                Self::stats(
                    &Self::frames(&self.frame_info[1][i].0),
                    &self.frame_info[1][i].1
                )
            );
            println!();
            println!("Next Fit:");
            println!(
                "[{}]",
                Self::draw_ram(&Self::frames(&self.frame_info[0][i].0))
            );
            println!(
                "{}",
                Self::stats(
                    &Self::frames(&self.frame_info[0][i].0),
                    &self.frame_info[0][i].1
                )
            );
            println!();
            println!("Worst Fit:");
            println!(
                "[{}]",
                Self::draw_ram(&Self::frames(&self.frame_info[2][i].0))
            );
            println!(
                "{}",
                Self::stats(
                    &Self::frames(&self.frame_info[2][i].0),
                    &self.frame_info[2][i].1
                )
            );
            if buff.trim().to_lowercase() == "y" {
                std::thread::sleep(Duration::from_secs(2));
            } else {
                println!("Please press enter to advance.");
                buff.clear();
                std::io::stdin().read_line(&mut buff).unwrap();
            }
        }
    }
}

pub(crate) fn draw_gui(
    frame_info: [Vec<(Vec<MemoryRegion>, Vec<MemoryRequest>)>; 3],
    config: Config,
) {
    Gui::new(frame_info).draw_gui(config);
}
