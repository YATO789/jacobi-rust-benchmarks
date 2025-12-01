pub mod safe;

#[path = "unsafe"]
pub mod unsafe_impl {
    pub mod unsafe_semaphore;
    pub mod parallel_unsafe;
}
