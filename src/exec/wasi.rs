use std::fmt::Write;
use std::io::{Read, Seek};
use std::mem::ManuallyDrop;
use std::{fs::File, os::fd::FromRawFd};
use std::sync::{Arc, Mutex};

pub trait ReadWrite: Read + Write + Seek + Send + Sync + 'static {}

impl<IO: Read + Write + Seek + Send + Sync + 'static> ReadWrite for IO {}

#[derive(Default)]
pub struct WasiSnapshotPreview1 {
    pub file_table: Vec<Arc<Mutex<Box<ManuallyDrop<File>>>>>,
}

impl WasiSnapshotPreview1 {
    pub fn new() -> Self {
        unsafe {
            Self {
                file_table: vec![
                    Arc::new(Mutex::new(Box::new(ManuallyDrop::new(File::from_raw_fd(0))))),
                    Arc::new(Mutex::new(Box::new(ManuallyDrop::new(File::from_raw_fd(1))))),
                    Arc::new(Mutex::new(Box::new(ManuallyDrop::new(File::from_raw_fd(2))))),
                ],
            }
        }
    }
}
