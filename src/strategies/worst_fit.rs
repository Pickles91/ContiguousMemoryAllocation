use std::collections::VecDeque;

//.___   _____ __________________ __________________________    __________________
//|   | /     \\______   \_____  \\______   \__    ___/  _  \   \      \__    ___/
//|   |/  \ /  \|     ___//   |   \|       _/ |    | /  /_\  \  /   |   \|    |
//|   /    Y    \    |   /    |    \    |   \ |    |/    |    \/    |    \    |
//|___\____|__  /____|   \_______  /____|_  / |____|\____|__  /\____|__  /____|
//            \/                 \/       \/                \/         \/
// if this seems familiar it's cause 90% of this code is just the bestfit code.
// :)

use super::{MemAllocator, MemoryRegion, MemoryRequest, Pid};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct WorstFit {
    reqs: VecDeque<MemoryRequest>,
    mem: Vec<MemoryRegion>,
    time: u32,
}

impl WorstFit {
    #[allow(unused)]
    pub fn new(mem_size: u32) -> Self {
        Self {
            reqs: VecDeque::new(),
            mem: vec![
                MemoryRegion(None, 0),
                MemoryRegion(Some(Pid(super::FINAL_MEM_REGION_PID)), mem_size),
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
            .max_by_key(|&(_, size)| size)
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

impl MemAllocator for WorstFit {
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

    fn dealloc(&self, proc: super::Pid) -> Self {
        let mut out = self.clone();
        out.mem = out
            .mem
            .into_iter()
            .map(|mem| {
                if mem.0 == Some(proc) {
                    MemoryRegion(None, mem.1)
                } else {
                    mem
                }
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

#[cfg(test)]
mod tests {
    use crate::strategies::{MemAllocator, MemoryRegion, MemoryRequest, Pid};

    use super::WorstFit;

    // AHHHHHHH PRODUCTION QUALITY TESTING IN PROGRESS.
    // VERY HIGH QUALITY
    // MUCH REASSURANCE.
    // MANY WOW.
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
    fn basic_worst_fit_test() {
        let mut allocator = WorstFit::new(128);
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
                MemoryRegion(Some(Pid(1)), 15), // gap of 6
                MemoryRegion(Some(Pid(2)), 21),
                MemoryRegion(None, 22),
                MemoryRegion(Some(Pid(3)), 25),
                MemoryRegion(None, 128),
            ]
        );
    }
}
