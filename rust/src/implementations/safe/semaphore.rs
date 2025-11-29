use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use crate::grid::{Grid, ALPHA, DT, DX, N, M};

