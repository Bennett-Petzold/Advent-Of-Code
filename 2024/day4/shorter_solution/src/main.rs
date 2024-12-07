use std::{
    env::args,
    fs::File,
    io::{BufRead, BufReader},
};

fn main() {
    let lines: Vec<_> = BufReader::new(File::open(args().nth(1).unwrap()).unwrap())
        .lines()
        .map(|param| param.unwrap())
        .collect();
    part_1(&lines);
    part_2(&lines);
}

fn part_1<S: AsRef<str>>(lines: &[S]) {
    let xmas: Vec<char> = "XMAS".chars().collect();

    let sum: usize = (0..lines.len())
        .map(|y| {
            (0..lines[0].as_ref().len())
                .map(|x| search_pos(lines, (x, y).into(), &xmas))
                .sum::<usize>()
        })
        .sum();
    println!("{sum}")
}

fn part_2<S: AsRef<str>>(lines: &[S]) {
    let sum: usize = (1..(lines.len() - 1))
        .map(|y| {
            (1..(lines[0].as_ref().len() - 1))
                .map(|x| search_mas_cross(lines, (x, y).into()))
                .filter(|x| *x)
                .count()
        })
        .sum();
    println!("{sum}")
}

#[derive(Debug, Clone, Copy)]
pub struct Pos2D {
    pub x: usize,
    pub y: usize,
}

impl Pos2D {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    // -- START Directional calculations -- //

    pub fn down(&self) -> Option<Self> {
        Some(Self::new(self.x, self.y.checked_add(1)?))
    }

    pub fn down_right(&self) -> Option<Self> {
        Some(Self::new(self.x.checked_add(1)?, self.y.checked_add(1)?))
    }

    pub fn right(&self) -> Option<Self> {
        Some(Self::new(self.x.checked_add(1)?, self.y))
    }

    pub fn up_right(&self) -> Option<Self> {
        Some(Self::new(self.x.checked_add(1)?, self.y.checked_sub(1)?))
    }

    pub fn up(&self) -> Option<Self> {
        Some(Self::new(self.x, self.y.checked_sub(1)?))
    }

    pub fn up_left(&self) -> Option<Self> {
        Some(Self::new(self.x.checked_sub(1)?, self.y.checked_sub(1)?))
    }

    pub fn left(&self) -> Option<Self> {
        Some(Self::new(self.x.checked_sub(1)?, self.y))
    }

    pub fn down_left(&self) -> Option<Self> {
        Some(Self::new(self.x.checked_sub(1)?, self.y.checked_add(1)?))
    }

    // -- END Directional calculations -- //

    pub fn surrounding_pos(&self) -> impl Iterator<Item = Self> {
        [
            Some(Self::new(self.x + 1, self.y)),
            Some(Self::new(self.x + 1, self.y + 1)),
            Some(Self::new(self.x, self.y + 1)),
            // -- negative x -- //
            self.x.checked_sub(1).map(|x| Self::new(x, self.y)),
            self.x.checked_sub(1).map(|x| Self::new(x, self.y + 1)),
            // -- negative y -- //
            self.y.checked_sub(1).map(|y| Self::new(self.x, y)),
            self.y.checked_sub(1).map(|y| Self::new(self.x + 1, y)),
            // -- negative x and y -- //
            self.x
                .checked_sub(1)
                .and_then(|x| self.y.checked_sub(1).map(|y| Self::new(x, y))),
        ]
        .into_iter()
        .flatten()
    }

    pub fn surrounding_lines(&self) -> impl Iterator<Item = SurroundingLineIter> + use<'_> {
        [
            Self::down,
            Self::down_right,
            Self::right,
            Self::up_right,
            Self::up,
            Self::up_left,
            Self::left,
            Self::down_left,
        ]
        .into_iter()
        .map(|func| SurroundingLineIter::new(*self, func))
    }

    pub fn get_arr_char<S, A>(&self, arr: A) -> Option<char>
    where
        S: AsRef<str>,
        A: AsRef<[S]>,
    {
        arr.as_ref().get(self.y)?.as_ref().chars().nth(self.x)
    }
}

impl From<(usize, usize)> for Pos2D {
    fn from(value: (usize, usize)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

#[derive(Debug)]
pub struct SurroundingLineIter {
    cur_pos: Pos2D,
    change: fn(&Pos2D) -> Option<Pos2D>,
}

impl SurroundingLineIter {
    pub fn new(pos: Pos2D, change: fn(&Pos2D) -> Option<Pos2D>) -> Self {
        Self {
            cur_pos: pos,
            change,
        }
    }
}

impl Iterator for SurroundingLineIter {
    type Item = Pos2D;

    fn next(&mut self) -> Option<Self::Item> {
        self.cur_pos = (self.change)(&self.cur_pos)?;
        Some(self.cur_pos)
    }
}

fn search_pos<S>(array: &[S], pos: Pos2D, search: &[char]) -> usize
where
    S: AsRef<str>,
{
    if let Some(first_char) = search.first() {
        if pos.get_arr_char(array) == Some(*first_char) {
            let count = pos
                .surrounding_lines()
                .map(|mut line| {
                    for next_search in &search[1..] {
                        if let Some(next_found) = line.next() {
                            if Some(*next_search) != next_found.get_arr_char(array) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    true
                })
                .filter(|x| *x)
                .count();
            count
        } else {
            0
        }
    } else {
        0
    }
}

fn search_mas_cross<S>(array: &[S], pos: Pos2D) -> bool
where
    S: AsRef<str>,
{
    let mid = pos.get_arr_char(array) == Some('A');

    let check_corners = |up: Option<Pos2D>, down: Option<Pos2D>| {
        up.and_then(|up_left| Some((up_left, down?)))
            .map(|(up_left, down_right)| match up_left.get_arr_char(array) {
                Some('M') => down_right.get_arr_char(array) == Some('S'),
                Some('S') => down_right.get_arr_char(array) == Some('M'),
                // Upper left needs to be M or S
                _ => false,
            })
            == Some(true)
    };

    mid && check_corners(pos.up_left(), pos.down_right())
        && check_corners(pos.up_right(), pos.down_left())
}
