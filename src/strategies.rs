mod best_fit;
mod next_fit;
mod worst_fit;

#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
pub struct Pid(u32);

/// A Memory Region in the Memory.
/// The first field represents a process that owns
/// the region, if any. The second field represents
/// where it starts.
/// In order to know where it ends, check the next
/// memory regions start field (exclusive).
#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
pub struct MemoryRegion(Option<Pid>, u32);

/// A Memory Request that needs to be served by the Memory
/// allocator. It holds a PID that's requesting the memory,
/// as well as the size it's requesting in KB.
#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
pub struct MemoryRequest {
    process: Pid,
    size: u32,
}

/// This MemAllocator API is an immutable API. When working with
/// a MemAllocator, (e.g. doing an allocation) - it returns a new
/// instance of the allocator with the modifications applied.
/// Although from a performance and real-world perspective this API
/// doesn't actually make much sense (your computer isn't storing
/// the current copy of memory, along with previous copies of memory),
/// it makes sense from a simulation perspective since it allows you
/// to step back and fourth, and reason about the changes over time.
trait MemAllocator
where
    Self: Sized,
{
    /// initializes a memory request, which returns a new
    /// instance of the allocator with the request logged.
    fn request(&self, req: MemoryRequest) -> Self;
    /// returns a new instance of the memory allocator after
    /// the tick, as well as a copy of the inner working
    /// memory.
    fn tick(&self) -> (Vec<MemoryRegion>, Self);
}
