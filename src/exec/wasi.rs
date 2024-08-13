use std::{fs::File, os::fd::FromRawFd};

#[derive(Default)]
pub struct WasiSnapshotPreview1 {
    pub file_table: Vec<Option<Box<File>>>,
    pub file_path: Vec<Option<String>>,
}

impl WasiSnapshotPreview1 {
    pub fn new() -> Self {
        let current_dir = File::open(".").unwrap();
        unsafe {
            Self {
                file_table: vec![
                    Some(Box::new(File::from_raw_fd(0))),
                    Some(Box::new(File::from_raw_fd(1))),
                    Some(Box::new(File::from_raw_fd(2))),
                    Some(Box::new(current_dir)),
                ],
                file_path: vec![
                    None,
                    None,
                    None,
                    Some(".".to_string()),
                ]
            }
        }
    }
}
