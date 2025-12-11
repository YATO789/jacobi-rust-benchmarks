pub mod safe;

#[path = "unsafe"]
pub mod unsafe_impl {
    pub mod unsafe_atomic_counter;
    pub mod barrier_unsafe;
    pub mod rayon_unsafe;
    pub mod single_unsafe;
}
