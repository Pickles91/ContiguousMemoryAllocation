use std::collections::VecDeque;

use super::{MemAllocator, MemoryRegion, MemoryRequest, Pid, FINAL_MEM_REGION_PID};

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
            mem: vec![
                MemoryRegion(None, 0),
                MemoryRegion(Some((Pid(FINAL_MEM_REGION_PID), -1)), mem_size),
            ],
            time: 0,
            offset: 0,
        }
    }

    fn fullfill_reqs(mut self) -> Self {

         // Attempt to pop the front of the requests queue.
        let Some(req) = self.reqs.pop_front() else {
            // if we have no requests nothing to do.
            return self;
        };

        // Find a fitting memory region for the current request.
        let fitting_region = self.mem[self.offset..]
            .windows(2)
            .chain(self.mem[..self.offset + 1].windows(2))
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
            // we couldn't find one, so do the other requests,
            // and then exit out.
            let mut out = self.fullfill_reqs();
            out.reqs.push_front(req);
            return out;
        };

        // increment the offset by how much we moved.
        // this is the current index we want to insert into.
        self.offset += index_from_offset;
        self.offset %= self.mem.len() - 1;

        // when inserting into the memory region we have to be careful that the next memory region
        // does not end up having the same starting point as the one after it (e.g. the next
        // region should not have a memory size of 0). If we do... prune it out.
        self.mem.insert(
            self.offset,
            MemoryRegion(
                Some((req.process, req.lifetime as _)),
                self.mem[self.offset].1,
            ),
        );
        self.mem[self.offset + 1].1 += req.size;
        match self.mem.get(self.offset + 2) {
            Some(MemoryRegion(_, starting)) if *starting == self.mem[self.offset + 1].1 => {
                self.mem.remove(self.offset + 1);
            }
            Some(MemoryRegion(_, starting)) if *starting < self.mem[self.offset + 1].1 => {
                dbg!((starting, self.mem[self.offset + 1].1));
                panic!("Overlapping memory regions")
            }
            _ => {}
        }
        // do the rest of the requests.
        self.fullfill_reqs()
    }

    /// Deallocates memory regions with zero size and merges neighboring regions
    fn dealloc(&self) -> Self {
        let mut out = self.clone();
        let offset_mem_addr = out.mem[self.offset].1;
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
        let (offset, region) = out
            .mem
            .iter()
            .enumerate()
            .filter(|(_, region)| region.1 >= offset_mem_addr)
            .next()
            .unwrap();
        out.offset = if region.1 as usize == self.offset {
            offset
        } else {
            offset - 1
        };
        out
    }
}

impl MemAllocator for NextFit {
    /// Handles a memory allocation request by adding it to the request queue.
    fn request(&self, req: MemoryRequest) -> Self {
        let mut out = self.clone();
        out.reqs.push_back(req);
        out
    }

    /// Advances the simulation by one time unit, updating memory regions' lifetimes
    /// and processing deallocation and request fulfillment.
    /// 
    /// Returns a tuple containing the current memory layout, processed requests, and
    /// the updated state of the memory allocator.
    fn tick(&self) -> (Vec<MemoryRegion>, Vec<MemoryRequest>, Self) {
        let mut out = self.clone();
        out.time += 1;
        for i in out.mem.iter_mut() {
            match i {
                MemoryRegion(Some((pid, lifetime)), _) if *lifetime > 0 => *lifetime -= 1,
                _ => {}
            }
        }
        let out = out.dealloc().fullfill_reqs();
        (out.mem.clone(), out.reqs.clone().into_iter().collect(), out)
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
                lifetime: 5,
            })
            .request(MemoryRequest {
                process: Pid(1),
                size: 11,
                lifetime: 5,
            })
            .request(MemoryRequest {
                process: Pid(2),
                size: 7,
                lifetime: 5,
            });
        let (mem, _, _) = allocator.tick();
        assert_eq!(
            mem,
            vec![
                MemoryRegion(Some((Pid(1), 5)), 0),
                MemoryRegion(Some((Pid(1), 5)), 10),
                MemoryRegion(Some((Pid(2), 5)), 21),
                MemoryRegion(None, 28),
                MemoryRegion(Some((Pid(FINAL_MEM_REGION_PID), -1)), 128)
            ]
        )
    }

    #[test]
    fn test_full() {
        let allocator = NextFit::new(128)
            .request(MemoryRequest {
                process: Pid(1),
                size: 100,
                lifetime: 5,
            })
            .request(MemoryRequest {
                process: Pid(2),
                size: 27,
                lifetime: 5,
            })
            .request(MemoryRequest {
                process: Pid(3),
                size: 13,
                lifetime: 5,
            });
        let (mem, _, _) = allocator.tick();
        assert_eq!(
            mem,
            vec![
                MemoryRegion(Some((Pid(1), 5)), 0),
                MemoryRegion(Some((Pid(2), 5)), 100),
                MemoryRegion(None, 127),
                MemoryRegion(Some((Pid(FINAL_MEM_REGION_PID), -1)), 128)
            ]
        )
    }

    /// I lied, here's another one. I wanted to test dealloc on atleast one impl,
    /// so here you go.
    #[test]
    fn basic_test_dealloc() {
        let (_, _, alloc) = NextFit::new(128)
            .request(MemoryRequest {
                process: Pid(1),
                size: 10,
                lifetime: 1,
            })
            .request(MemoryRequest {
                process: Pid(2),
                size: 7,
                lifetime: 1,
            })
            .tick();
        assert_eq!(
            alloc
                .request(MemoryRequest {
                    process: Pid(3),
                    size: 3,
                    lifetime: 5,
                })
                .tick()
                .0,
            vec![
                MemoryRegion(Some((Pid(3), 5)), 0),
                MemoryRegion(None, 3),
                MemoryRegion(Some((Pid(FINAL_MEM_REGION_PID), -1)), 128)
            ]
        )
    }
}
