use std::{
    char,
    env::args,
    fs::File,
    io::{BufRead, BufReader},
    str::Chars,
};

fn main() {
    let lines: Vec<_> = BufReader::new(File::open(args().nth(1).unwrap()).unwrap())
        .lines()
        .map(|line| line.unwrap())
        .collect();
    part_1(&lines);
}

fn part_1(lines: &[String]) {
    let sum = find_pattern(lines, "XMAS");
    println!("{sum}");
}

#[derive(Debug)]
struct RightDownDiagonal<I> {
    iter: I,
    pos: usize,
}

impl<I: Iterator<Item = T>, T: Iterator<Item = U>, U> RightDownDiagonal<I> {
    pub fn new(iter: I) -> Self {
        Self { iter, pos: 0 }
    }
}

impl<I: Iterator<Item = T>, T: Iterator<Item = U>, U> Iterator for RightDownDiagonal<I> {
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next()?.nth(self.pos)?;
        self.pos += 1;
        Some(item)
    }
}

#[derive(Debug)]
struct LeftDownDiagonal<I> {
    iter: I,
    pos: Option<usize>,
}

impl<I: Iterator<Item = T>, T: Iterator<Item = U>, U> LeftDownDiagonal<I> {
    pub fn new(iter: I, length: usize) -> Self {
        Self {
            iter,
            pos: Some(length - 1),
        }
    }
}

impl<I: Iterator<Item = T>, T: Iterator<Item = U>, U> Iterator for LeftDownDiagonal<I> {
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = &mut self.pos?;
        let item = self.iter.next()?.nth(*pos)?;

        if *pos > 0 {
            *pos -= 1;
        } else {
            self.pos = None;
        }

        Some(item)
    }
}

#[derive(Debug)]
struct DownSlices<'a> {
    slice: &'a [&'a str],
    pos: usize,
}

impl<'a> DownSlices<'a> {
    pub fn new(slice: &'a [&str]) -> Self {
        Self { slice, pos: 0 }
    }
}

impl<'a> Iterator for DownSlices<'a> {
    type Item = &'a [&'a str];
    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.len() < self.pos {
            let subslice = &self.slice[self.pos..];
            self.pos += 1;
            Some(subslice)
        } else {
            None
        }
    }
}

fn find_pattern<S: AsRef<str>>(lines: &[S], pattern: &str) -> u64 {
    let pattern: Vec<_> = pattern.chars().collect();
    let pattern = pattern.into_boxed_slice();

    let mut sum = find_pattern_axis(lines.iter().map(|x| x.as_ref().chars()), &pattern);
    sum += find_pattern_axis(lines.iter().rev().map(|x| x.as_ref().chars()), &pattern);
    /*
    sum += find_pattern_axis(
        LeftDownDiagonal::new(lines.iter().map(|x| x.as_ref().chars())),
        &pattern,
    );
    */

    sum
}

#[derive(Debug)]
struct StringBuffer<I> {
    iter: I,
    buf: Box<[char]>,
}

impl<I: Iterator<Item = char>> StringBuffer<I> {
    /// Initializes self if the iterator has least `len` characters.
    pub fn new(mut iter: I, len: usize) -> Option<Self> {
        let buf: Vec<_> = iter.by_ref().take(len).collect();
        let buf = buf.into_boxed_slice();
        if buf.len() == len {
            Some(Self { iter, buf })
        } else {
            None
        }
    }

    /// Returns self if the iterator has a next element.
    pub fn step(mut self) -> Option<Self> {
        let next_char = self.iter.by_ref().next()?;
        self.buf.copy_within(0..(self.buf.len() - 1), 1);
        self.buf[0] = next_char;

        Some(Self {
            iter: self.iter,
            buf: self.buf,
        })
    }
}

impl<I> AsRef<[char]> for StringBuffer<I> {
    fn as_ref(&self) -> &[char] {
        &self.buf
    }
}

fn find_pattern_axis<I, C>(axis: I, pattern: &[char]) -> u64
where
    I: Iterator<Item = C>,
    C: Iterator<Item = char>,
{
    let mut reverse_pattern = pattern.to_vec().into_boxed_slice();
    reverse_pattern.reverse();

    axis.map(|line| {
        let mut buffer = StringBuffer::new(line, pattern.len());
        let mut sub_sum = 0;
        while let Some(buf) = buffer {
            // Forward check
            if buf.as_ref() == pattern {
                sub_sum += 1
            }
            // Reverse check
            if *buf.as_ref() == *reverse_pattern {
                sub_sum += 1
            }
            // Rotate in next item, if there is one
            buffer = buf.step();
        }
        sub_sum
    })
    .sum()
}
