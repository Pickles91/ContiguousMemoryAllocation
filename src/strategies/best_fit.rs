use std::collections::VecDeque;

use super::{MemAllocator, MemoryRegion, MemoryRequest, Pid};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct BestFit {
    reqs: VecDeque<MemoryRequest>,
    mem: Vec<MemoryRegion>,
    time: u32,
}

impl BestFit {
    #[allow(unused)]
    pub fn new(mem_size: u32) -> Self {
        Self {
            reqs: VecDeque::new(),
            mem: vec![
                MemoryRegion(None, 0),
                MemoryRegion(Some((Pid(super::FINAL_MEM_REGION_PID), -1)), mem_size),
            ],
            time: 0,
        }
    }
    fn fullfill_reqs(mut self) -> Self {
        let Some(req) = self.reqs.pop_front() else {
            return self;
        };
        let Some((index, size)) = self
            .mem
            .windows(2)
            .map(|window| TryInto::<[MemoryRegion; 2]>::try_into(window).unwrap())
            .enumerate()
            .filter(|(_, [a, _])| a.0.is_none())
            .map(|(i, [a, b])| (i, b.1 - a.1))
            .filter(|&(_, size)| req.size <= size)
            .min_by_key(|&(_, size)| size)
        else {
            let mut out = self.fullfill_reqs();
            out.reqs.push_front(req);
            return out;
        };
        self.mem.insert(
            index,
            MemoryRegion(Some((req.process, req.lifetime as i32)), self.mem[index].1),
        );
        self.mem[index + 1].1 += size;
        match self.mem.get(index + 2) {
            Some(region) if region.1 == self.mem[index + 1].1 => {
                self.mem.remove(index + 1);
            }
            _ => {}
        };
        self
    }
    fn dealloc(&self) -> Self {
        let mut out = self.clone();
        out.mem = out
            .mem
            .into_iter()
            .map(|mem| match mem.0 {
                Some((_, 0)) => MemoryRegion(None, mem.1),
                _ => mem,
            })
            .collect();
        // merge neighboring regions with the same
        // owner by removing the second region with the same owner.
        out.mem = out.mem.windows(2).fold(vec![], |mut acc, regions| {
            let [prev, next]: [_; 2] = regions.try_into().unwrap();
            if prev.1 == 0 {
                acc.push(prev);
            }
            if next.0 != prev.0 {
                acc.push(next);
            }
            acc
        });
        out
    }
}

impl MemAllocator for BestFit {
    fn request(&self, req: MemoryRequest) -> Self {
        let mut out = self.clone();
        out.reqs.push_back(req);
        out
    }

    fn tick(&self) -> (Vec<MemoryRegion>, Self) {
        let mut out = self.clone();
        out.time += 1;
        for i in out.mem.iter_mut() {
            match i {
                MemoryRegion(Some((pid, lifetime)), _) if *lifetime > 0 => *lifetime -= 1,
                _ => {}
            }
        }
        let out = out.dealloc().fullfill_reqs();
        (out.mem.clone(), out)
    }
}

#[cfg(test)]
mod tests {
    use crate::strategies::{MemAllocator, MemoryRegion, MemoryRequest, Pid};

    use super::BestFit;

    /// AHHHHHHH PRODUCTION QUALITY TESTING IN PROGRESS.
    /// VERY HIGH QUALITY
    /// MUCH REASSURANCE.
    /// MANY WOW.
    fn foo() {}
    // ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢠⣴⣷⣦⠙⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠟⣛⡛⢿⣿⣿
    // ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢸⣿⣿⣿⣷⡸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢟⣡⣿⣿⡟⢣⠙⣿
    // ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢼⣿⣿⣿⣿⣷⡈⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢟⣵⠟⣿⣿⣿⣿⡄⢧⢸
    // ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⣼⣿⣿⣿⣿⣿⣿⣦⡉⠿⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⢟⣥⡿⢃⣼⣿⣿⣿⣿⣇⠸⡌
    // ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁⣿⠿⣩⣿⣿⣿⣿⣶⣿⣿⣷⣶⣤⣤⢤⡤⣤⡌⢉⣩⣴⣿⡟⢡⣿⣿⣿⣯⣹⣿⣿⠀⡇
    // ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⣸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣄⠙⠿⠋⢠⣿⣿⡉⠉⠁⢀⣿⣿⠀⡇
    // ⣿⣿⣿⣿⣿⣿⣿⣿⡿⠛⣡⣾⣿⣿⣿⣿⣿⣿⣿⣿⢟⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠰⣿⣿⣿⠉⠀⠀⢸⣿⣿⡰⢡
    // ⣿⣿⣿⣿⣿⣿⡿⢋⣤⣿⣿⣿⣿⣿⣿⣿⣿⡿⡿⢁⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣧⡙⠛⠁⠀⠀⣴⣿⣿⢿⠁⣾
    // ⣿⣿⣿⣿⣿⠟⣠⣾⣿⣿⣿⠿⣿⣿⣿⣿⡿⢠⣷⡿⠛⣽⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣶⣦⠰⢾⣿⣯⣥⣼⠀⣿
    // ⣿⣿⣿⣿⠋⣴⣿⣿⣿⢋⠀⠀⢠⠈⣿⣿⣵⣿⣋⣤⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣦⣿⣿⣿⣿⡆⢹
    // ⣿⣿⣿⠋⣼⣿⣿⣿⣿⠸⠀⠀⠀⣼⣿⣿⣿⣿⣿⣿⣿⣿⡿⠛⢉⡉⠉⠻⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢸
    // ⣿⣿⠇⣼⣿⣿⣿⡟⣿⣵⣤⣶⣾⣿⣿⣿⣿⣿⣿⣿⡍⠁⠀⢰⠁⠀⠀⠀⠀⣩⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡘
    // ⣿⡟⢸⣿⣿⣿⣿⣷⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣷⣤⣀⠂⠀⢀⣰⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢣
    // ⡿⢀⣿⣿⣿⣿⡿⠛⠉⠠⠉⠙⠻⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢸
    // ⡇⣸⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠘⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢸
    // ⠃⣿⡿⢿⣿⣿⡈⠀⠀⠀⠀⠀⠀⢀⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⢸
    // ⢸⣿⣷⣿⣿⣿⠓⣄⠀⠀⠀⢀⣦⣿⣿⡿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⣾
    // ⡆⢿⣿⣿⣿⣷⠀⠉⠀⠀⠢⢤⡌⣒⣲⣶⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡏⣼⣿
    // ⣇⢸⣿⣿⣿⣿⣦⠀⠀⠀⠀⠀⠀⠈⠙⠿⠿⠿⠛⠛⠋⢉⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠏⣸⣿⣿
    // ⣿⡀⢻⣿⣿⣿⣿⣿⣦⠸⣶⣶⣦⣤⣄⣠⣤⣤⣶⣦⣶⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⢏⣴⣿⣿⣿
    // ⣿⣷⡘⣿⣿⣿⣿⣿⣿⣷⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠟⣡⣾⣿⣿⣿⣿
    // ⣿⣿⣧⡈⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠟⣡⣾⣿⣿⣿⣿⣿⣿
    // ⣿⣿⣿⣿⣎⡹⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⢋⣴⣾⣿⣿⣿⣿⣿⣿⣿⣿
    // ⣿⣿⣿⣿⣿⣿⣤⡙⠿⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⠟⣋⣠⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
    // ⣿⣿⣿⣿⣿⣿⣿⣿⣷⣤⣍⣛⠛⠛⠛⠻⣿⣿⣿⣿⣿⣿⣿⡟⠛⠛⣛⣋⣭⣤⣴⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
    #[test]
    fn basic_best_fit_test() {
        let mut allocator = BestFit::new(128);
        allocator.mem = vec![
            MemoryRegion(Some((Pid(0), 3)), 0),
            MemoryRegion(None, 15), // gap of 6
            MemoryRegion(Some((Pid(2), 3)), 21),
            MemoryRegion(None, 22), // gap of 3
            MemoryRegion(Some((Pid(3), 3)), 25),
            MemoryRegion(None, 128),
        ];
        assert_eq!(
            allocator
                .request(MemoryRequest {
                    process: Pid(1),
                    size: 3,
                    lifetime: 3,
                })
                .tick()
                .0,
            vec![
                MemoryRegion(Some((Pid(0), 2)), 0),
                MemoryRegion(None, 15), // gap of 6
                MemoryRegion(Some((Pid(2), 2)), 21),
                MemoryRegion(Some((Pid(1), 3)), 22),
                MemoryRegion(Some((Pid(3), 2)), 25),
                MemoryRegion(None, 128),
            ]
        );
    }
}
