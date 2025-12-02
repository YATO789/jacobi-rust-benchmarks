pub mod safe;

#[path = "unsafe"]
pub mod unsafe_impl {
    pub mod unsafe_semaphore;
    pub mod barrier_unsafe;
    pub mod rayon_unsafe;
}
