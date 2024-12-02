#[derive(Debug)]
pub struct IndexHeader {
    num_entries: u32,   // 3
    version: u32,       // 2
    signature: [u8; 4], // 1
}

impl IndexHeader {
    pub fn new(num_entries: u32, version: u32, signature: [u8; 4]) -> Self {
        Self {
            num_entries,
            version,
            signature,
        }
    }

    pub fn num_entries(&self) -> u32 {
        self.num_entries
    }

    pub fn set_num_entries(&mut self, num: u32) {
        self.num_entries = num;
    }

    pub fn increment_num_entries(&mut self) {
        self.num_entries += 1;
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn signature(&self) -> [u8; 4] {
        self.signature
    }
}

#[derive(Debug)]
pub struct IndexEntry {
    pub ctime: (u32, u32), // when file's metadata was last changed (seconds and nanoseconds)
    pub mtime: (u32, u32), // when file's content was last modified (seconds and nanoseconds)
    pub device: u32,
    pub inode: u32,
    pub mode: u32,
    pub size: u32,
    pub sha_hash: Vec<u8>, // SHA 256
    pub file_path_len: u32,
    pub file_path: String, // Relative path of file from root, stored as null-terminated str
}
