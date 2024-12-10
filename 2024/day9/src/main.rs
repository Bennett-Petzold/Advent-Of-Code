use advent_rust_lib::read::input;

fn main() {
    let disk = Disk::from_input_line(input().next().unwrap());
    part_1(disk.clone());
    part_2(disk);
}

fn part_1(mut disk: Disk) {
    disk.pack_in_order();

    #[cfg(feature = "print_disk")]
    disk.print();

    let sum: u64 = disk
        .pos_id_iter()
        .map(|(x, y)| (x as u64) * (y as u64))
        .sum();
    println!("{sum}")
}

fn part_2(mut disk: Disk) {
    disk.pack_contiguous_in_order();

    #[cfg(feature = "print_disk")]
    disk.print();

    let sum: u64 = disk
        .pos_id_iter()
        .map(|(x, y)| (x as u64) * (y as u64))
        .sum();
    println!("{sum}")
}

// -------------------------------------------------- //

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileID(u32);

impl From<FileID> for u32 {
    fn from(val: FileID) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiskType {
    FreeSpace,
    File(FileID),
}

/// A chunk of disk, either a file or empty space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiskChunk {
    pub ty: DiskType,
    pub size: u32,
}

/// A chunk explicitly known to be a file
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileChunk {
    pub id: FileID,
    pub size: u32,
}

impl From<FileChunk> for DiskChunk {
    fn from(value: FileChunk) -> Self {
        Self {
            ty: DiskType::File(value.id),
            size: value.size,
        }
    }
}

impl DiskChunk {
    pub fn new(ty: DiskType, size: u32) -> Self {
        Self { ty, size }
    }

    pub fn as_file(self) -> Option<FileChunk> {
        if let DiskType::File(id) = self.ty {
            Some(FileChunk {
                id,
                size: self.size,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Disk {
    chunks: Vec<DiskChunk>,
}

impl Disk {
    fn from_input_line<S: AsRef<str>>(line: S) -> Self {
        let mut file = true;
        let mut file_id = 0;
        let mut chunks: Vec<_> = line
            .as_ref()
            .chars()
            .filter_map(|size| size.to_digit(10))
            .map(|size| {
                let cur_file = file;
                file = !file;
                DiskChunk::new(
                    if cur_file {
                        let cur_file_id = file_id;
                        file_id += 1;
                        DiskType::File(FileID(cur_file_id))
                    } else {
                        DiskType::FreeSpace
                    },
                    size,
                )
            })
            .filter(|file| file.size != 0)
            .collect();

        // Trim empty space from end
        {
            let trailing_space = chunks
                .iter()
                .rev()
                .take_while(|x| x.ty == DiskType::FreeSpace)
                .count();

            if trailing_space > 0 {
                chunks.truncate(chunks.len() - trailing_space);
            }
        }

        Self { chunks }
    }

    fn pack_in_order(&mut self) {
        let mut last_item = None;

        let mut idx = 0;
        while idx < self.chunks.len() {
            let free_chunk = self.chunks[idx];
            if free_chunk.ty == DiskType::FreeSpace {
                while last_item.is_none() && idx < self.chunks.len() {
                    last_item = self.chunks.pop().and_then(|chunk| chunk.as_file());
                }

                if let Some(last_item_res) = last_item.as_mut() {
                    self.chunks[idx].ty = DiskType::File(last_item_res.id);

                    match last_item_res.size.cmp(&free_chunk.size) {
                        std::cmp::Ordering::Equal => {
                            last_item = None;
                        }
                        std::cmp::Ordering::Greater => {
                            last_item_res.size -= free_chunk.size;
                        }
                        std::cmp::Ordering::Less => {
                            self.chunks[idx].size = last_item_res.size;

                            let leftover_size = free_chunk.size - last_item_res.size;
                            self.chunks.insert(
                                idx + 1,
                                DiskChunk::new(DiskType::FreeSpace, leftover_size),
                            );

                            last_item = None;
                        }
                    };
                }
            }

            // Increment to check next chunk
            idx += 1;
        }

        // Drop trailing free space
        if idx < self.chunks.len() {
            self.chunks.truncate(idx);
        }

        if let Some(last_item) = last_item {
            self.chunks.push(last_item.into());
        }
    }

    fn pack_contiguous_in_order(&mut self) {
        if let Some(highest_id) = self.chunks.last().map(|x| {
            x.as_file()
                .expect("Last entry is always a file, by construction")
                .id
        }) {
            // Test each candidate once, highest to lowest
            // 0 can never shift left, so it is skipped
            for id in (1..=highest_id.into()).rev().map(FileID) {
                let (mut file_chunk_idx, file_chunk) = self
                    .chunks
                    .iter()
                    .enumerate()
                    .rev()
                    .flat_map(|(idx, chunk)| Some((idx, chunk.as_file()?)))
                    .find(|(_, file)| file.id == id)
                    .expect("Chunks are never removed");

                let first_free = (0..file_chunk_idx)
                    .flat_map(|free_idx| {
                        let free_chunk = self.chunks[free_idx];

                        if (free_chunk.ty == DiskType::FreeSpace)
                            && (file_chunk.size <= free_chunk.size)
                        {
                            Some((free_idx, free_chunk))
                        } else {
                            None
                        }
                    })
                    .next();

                if let Some((free_idx, free_chunk)) = first_free {
                    self.chunks[free_idx].ty = DiskType::File(file_chunk.id);

                    if file_chunk.size < free_chunk.size {
                        self.chunks[free_idx].size = file_chunk.size;

                        let leftover_size = free_chunk.size - file_chunk.size;
                        self.chunks.insert(
                            free_idx + 1,
                            DiskChunk::new(DiskType::FreeSpace, leftover_size),
                        );
                        file_chunk_idx += 1;
                    }

                    match (
                        self.chunks[file_chunk_idx - 1].ty == DiskType::FreeSpace,
                        self.chunks
                            .get(file_chunk_idx + 1)
                            .map(|x| x.ty == DiskType::FreeSpace)
                            == Some(true),
                    ) {
                        (true, true) => {
                            self.chunks[file_chunk_idx - 1].size += self
                                .chunks
                                .drain(file_chunk_idx..=(file_chunk_idx + 1))
                                .map(|x| x.size)
                                .sum::<u32>();
                        }
                        (false, true) => {
                            self.chunks[file_chunk_idx].ty = DiskType::FreeSpace;

                            self.chunks[file_chunk_idx].size +=
                                self.chunks[file_chunk_idx + 1].size;
                            self.chunks.remove(file_chunk_idx + 1);
                        }
                        (true, false) => {
                            self.chunks[file_chunk_idx - 1].size +=
                                self.chunks[file_chunk_idx].size;
                            self.chunks.remove(file_chunk_idx);
                        }
                        (false, false) => {
                            self.chunks[file_chunk_idx].ty = DiskType::FreeSpace;
                        }
                    }
                }
            }
        }

        // Trim empty space from end
        {
            let trailing_space = self
                .chunks
                .iter()
                .rev()
                .take_while(|x| x.ty == DiskType::FreeSpace)
                .count();

            if trailing_space > 0 {
                self.chunks.truncate(self.chunks.len() - trailing_space);
            }
        }
    }

    /// Return (position, id) pairs
    pub fn pos_id_iter(&self) -> DiskFileIter<'_> {
        DiskFileIter::new(&self.chunks)
    }

    /// Print this representation to stdout
    #[cfg(feature = "print_disk")]
    pub fn print(&self) {
        for chunk in &self.chunks {
            let mut print_str = match chunk.ty {
                DiskType::FreeSpace => ".".to_string(),
                DiskType::File(x) => u32::from(x).to_string(),
            };

            print_str.push(' ');
            if chunk.size > 1 {
                print_str = print_str.repeat(chunk.size as usize);
            }

            print!("{print_str}");
        }
        println!();
    }
}

#[derive(Debug)]
pub struct DiskFileIter<'a> {
    chunks: std::slice::Iter<'a, DiskChunk>,
    position: u32,
    cur_chunk: Option<FileChunk>,
}

impl<'a> DiskFileIter<'a> {
    pub fn new(chunks: &'a [DiskChunk]) -> Self {
        let mut chunks = chunks.iter();

        let mut position = 0;

        for next_chunk in chunks.by_ref() {
            if let Some(as_file) = next_chunk.as_file() {
                return Self {
                    chunks,
                    position,
                    cur_chunk: Some(as_file),
                };
            } else {
                position += next_chunk.size;
            }
        }

        Self {
            chunks,
            position,
            cur_chunk: None,
        }
    }
}

impl Iterator for DiskFileIter<'_> {
    /// (position, id) pair
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk_under_test = self.cur_chunk.take();

        chunk_under_test.as_mut().map(|x| {
            let cur_position = self.position;
            self.position += 1;

            x.size -= 1;
            if x.size == 0 {
                for next_chunk in self.chunks.by_ref() {
                    if let Some(as_file) = next_chunk.as_file() {
                        self.cur_chunk = Some(as_file);
                        break;
                    } else {
                        self.position += next_chunk.size;
                    }
                }
            } else {
                self.cur_chunk = Some(*x);
            }

            (cur_position, x.id.into())
        })
    }
}
