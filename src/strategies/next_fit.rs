use std::collections::VecDeque;

use super::{MemAllocator, MemoryRegion, MemoryRequest};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct NextFit {
    reqs: VecDeque<MemoryRequest>,
    mem: Vec<MemoryRegion>,
    // offset into the memory, which is where it's
    // last gone and searched through.
    offset: usize,
    time: u32,
}

impl NextFit {
    #[allow(unused)]
    pub fn new(mem_size: u32) -> Self {
        Self {
            reqs: VecDeque::new(),
            mem: vec![MemoryRegion(None, 0), MemoryRegion(None, mem_size)],
            time: 0,
            offset: 0,
        }
    }
    fn fullfill_reqs(mut self) -> Self {
        let Some(req) = self.reqs.pop_front() else {
            // if we have no requests nothing to do.
            return self;
        };
        let fitting_region = self.mem[self.offset..]
            .windows(2)
            .chain(self.mem[..self.offset].windows(2))
            .enumerate()
            .find(|&(_, item)| {
                let [a, b]: [MemoryRegion; 2] = item.try_into().unwrap();
                if a.0.is_some() {
                    // this memory region belongs to a process, we can't allocate here.
                    return false;
                }
                if b.1 - a.1 < req.size {
                    // this memory region is too small.
                    return false;
                }
                true
            });
        let Some((index_from_offset, _)) = fitting_region else {
            // we couldn't find one, so do the next request.
            let mut out = self.fullfill_reqs();
            out.reqs.push_front(req);
            return out;
        };

        // increment the offset by how much we moved.
        // this is the current index we want to insert into.
        self.offset += index_from_offset;
        self.offset %= self.mem.len();

        // Insert the new memory region, followed by modifying the next memory region to be smaller (or be gone, depending).
        self.mem.insert(
            self.offset,
            MemoryRegion(Some(req.process), self.mem[self.offset].1),
        );
        self.mem[self.offset + 1].1 += req.size;
        match self.mem.get(self.offset + 2) {
            Some(MemoryRegion(_, starting)) if *starting <= self.mem[self.offset + 1].1 => {
                self.mem.remove(self.offset + 1);
            }
            _ => {}
        }
        // do the rest of the requests.
        self.fullfill_reqs()
    }
}

impl MemAllocator for NextFit {
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
    use crate::strategies::Pid;

    use super::*;

    /// I'm too lazy to do proper testing, but I wanna make sure it still
    /// works so I'll throw in a single unit test for your reading pleasure.
    /// Please note, I'm not liable if you decide to trust this unit test
    /// as proof this garbage I wrote works and you threw it in some poor sap's
    /// pace maker making them kick the bucket.
    #[test]
    fn basic_next_fit_test() {
        let allocator = NextFit::new(128)
            .request(MemoryRequest {
                process: Pid(1),
                size: 10,
            })
            .request(MemoryRequest {
                process: Pid(1),
                size: 11,
            })
            .request(MemoryRequest {
                process: Pid(2),
                size: 7,
            });
        let (mem, _) = allocator.tick();
        assert_eq!(
            mem,
            vec![
                MemoryRegion(Some(Pid(1)), 0),
                MemoryRegion(Some(Pid(1)), 10),
                MemoryRegion(Some(Pid(2)), 21),
                MemoryRegion(None, 28),
                MemoryRegion(None, 128)
            ]
        )
    }
}
