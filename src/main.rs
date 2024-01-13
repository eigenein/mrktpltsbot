#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

#[global_allocator]
static ALLOCATOR: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {}
