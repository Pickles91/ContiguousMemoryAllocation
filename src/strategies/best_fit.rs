use std::collections::VecDeque;

use super::{MemAllocator, MemoryRegion, MemoryRequest};

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
            mem: vec![MemoryRegion(None, 0), MemoryRegion(None, mem_size)],
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
        self.mem
            .insert(index, MemoryRegion(Some(req.process), self.mem[index].1));
        self.mem[index + 1].1 += size;
        match self.mem.get(index + 2) {
            Some(region) if region.1 == self.mem[index + 1].1 => {
                self.mem.remove(index + 1);
            }
            _ => {}
        };
        self
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
        let out = out.fullfill_reqs();
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
            MemoryRegion(Some(Pid(0)), 0),
            MemoryRegion(None, 15), // gap of 6
            MemoryRegion(Some(Pid(2)), 21),
            MemoryRegion(None, 22), // gap of 3
            MemoryRegion(Some(Pid(3)), 25),
            MemoryRegion(None, 128),
        ];
        assert_eq!(
            allocator
                .request(MemoryRequest {
                    process: Pid(1),
                    size: 3
                })
                .tick()
                .0,
            vec![
                MemoryRegion(Some(Pid(0)), 0),
                MemoryRegion(None, 15), // gap of 6
                MemoryRegion(Some(Pid(2)), 21),
                MemoryRegion(Some(Pid(1)), 22),
                MemoryRegion(Some(Pid(3)), 25),
                MemoryRegion(None, 128),
            ]
        );
    }
}
