use std::io::{self, Read, Write};
use std::{fs, path::Path};

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

fn extract_header(b: &Vec<u8>) -> io::Result<IndexHeader> {
    // first 4 bytes are the signature
    if &b[..4] != b"DIRC" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid INDEX file format",
        ));
    }

    let version = u32::from_be_bytes(b[4..8].try_into().unwrap());
    let num_entries = u32::from_be_bytes(b[8..12].try_into().unwrap());

    Ok(IndexHeader {
        num_entries,
        version,
        signature: *b"DIRC",
    })
}

fn create_index_entry_from_bytes(buffer: &Vec<u8>, offset: usize) -> (IndexEntry, usize) {
    let ctime_sec = u32::from_be_bytes(buffer[offset..offset + 4].try_into().unwrap());
    let ctime_nsec = u32::from_be_bytes(buffer[offset + 4..offset + 8].try_into().unwrap());
    let mtime_sec = u32::from_be_bytes(buffer[offset + 8..offset + 12].try_into().unwrap());
    let mtime_nsec = u32::from_be_bytes(buffer[offset + 12..offset + 16].try_into().unwrap());
    let device = u32::from_be_bytes(buffer[offset + 16..offset + 20].try_into().unwrap());
    let inode = u32::from_be_bytes(buffer[offset + 20..offset + 24].try_into().unwrap());
    let mode = u32::from_be_bytes(buffer[offset + 24..offset + 28].try_into().unwrap());
    let size = u32::from_be_bytes(buffer[offset + 28..offset + 32].try_into().unwrap());
    let sha_hash = buffer[offset + 32..offset + 64].to_vec();

    let mut pos = offset + 64;
    let file_path_len = u32::from_be_bytes(buffer[pos..pos + 4].try_into().unwrap());
    pos += 4;

    let file_path = String::from_utf8_lossy(&buffer[pos..pos + file_path_len as usize]).to_string();
    pos += file_path_len as usize;
    (
        IndexEntry {
            ctime: (ctime_sec, ctime_nsec),
            mtime: (mtime_sec, mtime_nsec),
            device,
            inode,
            mode,
            size,
            sha_hash,
            file_path_len,
            file_path,
        },
        pos,
    )
}

pub fn read_index() -> io::Result<(IndexHeader, Vec<IndexEntry>)> {
    let index_path = Path::new("./.rit/INDEX");
    let mut f = fs::File::open(&index_path)?;

    let mut buffer = vec![];
    f.read_to_end(&mut buffer)?;

    let header = extract_header(&buffer)?;

    let mut entries = vec![];
    let mut offset = 12;

    for _ in 0..header.num_entries {
        let (ie, pos) = create_index_entry_from_bytes(&buffer, offset);
        println!("entry: {:?}", ie);
        entries.push(ie);

        // Align to 8-byte boundary
        offset = pos + 1;
        offset = (offset + 7) & !7;
    }

    Ok((header, entries))
}

pub fn add_index(index_header: IndexHeader, index_entries: Vec<IndexEntry>) -> io::Result<bool> {
    let index_path = Path::new("./.rit/INDEX");
    let mut f = if index_path.exists() {
        fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(index_path)?
    } else {
        fs::File::create(index_path)?
    };

    // println!("{:?}", index_header);
    // println!("{:?}", index_entries);

    if f.metadata().unwrap().len() == 0 {
        // Write header if the file is empty
        f.write(&index_header.signature())?;
        f.write(&index_header.version().to_be_bytes())?;
        f.write(&index_header.num_entries().to_be_bytes())?;
    }

    // Index
    for ie in index_entries {
        f.write(&ie.ctime.0.to_be_bytes())?;
        f.write(&ie.ctime.1.to_be_bytes())?;
        f.write(&ie.mtime.0.to_be_bytes())?;
        f.write(&ie.mtime.1.to_be_bytes())?;
        f.write(&ie.device.to_be_bytes())?;
        f.write(&ie.inode.to_be_bytes())?;
        f.write(&ie.mode.to_be_bytes())?;
        f.write(&ie.size.to_be_bytes())?;
        f.write(&ie.sha_hash[..])?;
        f.write(&ie.file_path_len.to_be_bytes())?;
        f.write(&ie.file_path.as_bytes())?;
    }

    Ok(true)
}
