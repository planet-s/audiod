use std::collections::{BTreeMap, VecDeque};
use syscall::error::{EBADF, Error, Result};
use syscall::scheme::SchemeBlockMut;

// The strict buffer size of the hda: driver
const HDA_BUFFER_SIZE: usize = 512;
// The desired buffer size of each handle
const HANDLE_BUFFER_SIZE: usize = 4096;

struct Handle {
    flags: usize,
    buffer: VecDeque<(u16, u16)>,
}

pub struct AudioScheme {
    next_id: usize,
    handles: BTreeMap<usize, Handle>
}

impl AudioScheme {
    pub fn new() -> Self {
        AudioScheme {
            next_id: 0,
            handles: BTreeMap::new()
        }
    }

    pub fn buffer(&mut self) -> [(u16, u16); HDA_BUFFER_SIZE] {
        let mut buffer = [(0, 0); HDA_BUFFER_SIZE];

        for (_id, handle) in self.handles.iter_mut() {
            let mut i = 0;
            while i < buffer.len() {
                if let Some(sample) = handle.buffer.pop_front() {
                    buffer[i].0 += sample.0;
                    buffer[i].1 += sample.1;
                } else {
                    break;
                }
                i += 1;
            }
        }

        buffer
    }
}

impl SchemeBlockMut for AudioScheme {
    fn open(&mut self, _path: &[u8], flags: usize, _uid: u32, _gid: u32) -> Result<Option<usize>> {
        self.next_id += 1;
        let id = self.next_id;

        self.handles.insert(id, Handle {
            flags,
            buffer: VecDeque::new()
        });

        Ok(Some(id))
    }

    fn write(&mut self, id: usize, buf: &[u8]) -> Result<Option<usize>> {
        let mut handle = self.handles.get_mut(&id).ok_or(Error::new(EBADF))?;

        if handle.buffer.len() >= HANDLE_BUFFER_SIZE {
            Ok(None)
        } else {
            let mut i = 0;
            while i + 4 <= buf.len() {
                handle.buffer.push_back((
                    (buf[i] as u16) | ((buf[i + 1] as u16) << 8),
                    (buf[i + 2] as u16) | ((buf[i + 3] as u16) << 8)
                ));

                i += 4;
            }

            Ok(Some(i))
        }
    }
}
