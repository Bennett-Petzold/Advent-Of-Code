use stdcur_pos
    cmp::Reverse,
    collections::{BinaryHeap, HashSet},
    env::args,
    iter::Map,
    rc::Rc,
};

use advent_rust_lib::{
    direction::Direction,
    grid::{Pos2D, RectangleGrid},
    read::filtered_input,
};
use rayon::iter::Rev;

fn main() {
    let mut arg_numbers = args()
        .skip(1)
        .flat_map(|line| str::parse::<usize>(&line).ok());
    let dim = arg_numbers.next().unwrap();
    let count = arg_numbers.next().unwrap();

    let mem_space = MemSpace::from_input(dim, count, filtered_input(&[3])).unwrap();
    part_1(&mem_space);

    let full_mem_space = FillingMemSpace::from_input(dim, filtered_input(&[3])).unwrap();
    part_2(full_mem_space);
}

fn part_1(mem_space: &MemSpace) {
    println!("{}", mem_space.shortest_path());
}
fn part_2<I: IntoIterator<Item = S>, S>(mut mem_space: FillingMemSpace<I, S>) {
    let first_invalid = mem_space.first_invalid_fill().unwrap();
    println!("{},{}", first_invalid.x, first_invalid.y);
}

#[derive(Debug)]
pub struct MemSpace {
    // True is an obstacle
    grid: RectangleGrid<bool>,
}

impl MemSpace {
    pub fn from_input<S, I>(dim: usize, count: usize, input: I) -> Option<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut grid = RectangleGrid::default_with_dim(dim, dim, false);

        let input = input.into_iter().map(|line| {
            let line = line.as_ref();
            let (x, y) = line.split_at(line.find(',')?);
            let x = str::parse(x).ok()?;
            let y = str::parse(&y[1..]).ok()?;
            Some(Pos2D::new(x, y))
        });

        for block_pos in input.take(count) {
            let block_pos = block_pos?;
            *grid.get_mut(block_pos)? = true;
        }

        Some(Self { grid })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SteppedPos {
    pub pos: Pos2D,
    pub steps: u64,
}

impl PartialOrd for SteppedPos {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SteppedPos {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.steps.cmp(&other.steps).then(self.pos.cmp(&other.pos))
    }
}

impl MemSpace {
    pub fn shortest_path(&self) -> u64 {
        let final_pos = Pos2D::new(self.grid.x_max() - 1, self.grid.y_max() - 1);

        let mut visited = HashSet::new();
        let mut to_visit = BinaryHeap::from([Reverse(SteppedPos {
            pos: Pos2D::new(0, 0),
            steps: 0,
        })]);

        while to_visit.peek().expect("always at least one element").0.pos != final_pos {
            let top = to_visit.pop().expect("always at least one element").0;
            if visited.contains(&top.pos) {
                continue;
            }
            visited.insert(top.pos);

            let new_visit_locs = Direction::all()
                .into_iter()
                .flat_map(|dir| top.pos.step_dir(dir))
                .filter(|new_pos| !visited.contains(new_pos))
                .filter(|new_pos| {
                    if let Some(blocked) = self.grid.get(*new_pos) {
                        !*blocked
                    } else {
                        false
                    }
                })
                .map(|new_pos| {
                    Reverse(SteppedPos {
                        pos: new_pos,
                        steps: top.steps + 1,
                    })
                });
            to_visit.extend(new_visit_locs);
        }

        to_visit
            .peek()
            .expect("always at least one element")
            .0
            .steps
    }
}

pub struct FillingMemSpace<I: IntoIterator<Item = S>, S> {
    // True is an obstacle
    grid: RectangleGrid<bool>,
    #[expect(clippy::type_complexity, reason = "broken apart iterator")]
    fill: Map<<I as IntoIterator>::IntoIter, fn(S) -> Option<Pos2D>>,
    path: Vec<Pos2D>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RevLinkedNodeInternal {
    value: Pos2D,
    next: RevLinkedNode,
}

pub type RevLinkedNode = Option<Rc<RevLinkedNodeInternal>>;

impl RevLinkedNodeInternal {
    pub fn new() -> RevLinkedNode {
        None
    }

    pub fn push(this: RevLinkedNode, new_pos: Pos2D) -> RevLinkedNode {
        Some(Rc::new(Self {
            value: new_pos,
            next: this,
        }))
    }

    pub fn iter(this: RevLinkedNode) -> RevLinkedNodeIter {
        RevLinkedNodeIter { node: this }
    }
}

// Iterates in reverse order
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RevLinkedNodeIter {
    node: RevLinkedNode,
}

impl Iterator for RevLinkedNodeIter {
    type Item = Pos2D;

    fn next(&mut self) -> Option<Self::Item> {
        let prev_node = std::mem::take(&mut self.node)?;
        let value = prev_node.value;
        self.node = prev_node.next.clone();

        Some(value)
    }
}

#[derive(Debug, Clone)]
struct MemorySteppedPos {
    pub pos: Pos2D,
    pub steps: u64,
    pub memory: RevLinkedNode,
}

impl PartialEq for MemorySteppedPos {
    fn eq(&self, other: &Self) -> bool {
        (self.pos == other.pos) && (self.steps == other.steps)
    }
}

impl Eq for MemorySteppedPos {}

impl PartialOrd for MemorySteppedPos {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MemorySteppedPos {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.steps.cmp(&other.steps).then(self.pos.cmp(&other.pos))
    }
}

impl<I: IntoIterator<Item = S>, S> FillingMemSpace<I, S> {
    fn line_transform(line: S) -> Option<Pos2D>
    where
        S: AsRef<str>,
    {
        let line = line.as_ref();
        let (x, y) = line.split_at(line.find(',')?);
        let x = str::parse(x).ok()?;
        let y = str::parse(&y[1..]).ok()?;
        Some(Pos2D::new(x, y))
    }

    pub fn from_input(dim: usize, input: I) -> Option<Self>
    where
        S: AsRef<str>,
    {
        let grid = RectangleGrid::default_with_dim(dim, dim, false);
        let fill = input.into_iter().map(Self::line_transform as fn(_) -> _);

        let y_steps = (0..dim - 1).map(|y| Pos2D::new(0, y));
        let x_steps = (1..dim - 1).map(|x| Pos2D::new(x, 0));
        let path = y_steps.chain(x_steps).collect();

        Some(Self { grid, fill, path })
    }

    pub fn first_invalid_fill(&mut self) -> Option<Pos2D> {
        loop {
            let next_fill = self.fill.next()??;
            *self.grid.get_mut(next_fill)? = true;
            if !self.validate(next_fill)? {
                return Some(next_fill);
            }
        }
    }

    fn validate(&mut self, next_fill: Pos2D) -> Option<bool> {
        if let Some(overlap_idx) = self.path.iter().position(|entry| *entry == next_fill) {
            // Retain the path up until the overlap and attempt to form a new completition
            self.path.truncate(overlap_idx);

            let mut new_paths = (1..self.path.len())
                .rev()
                .flat_map(|test_idx| {
                    Some((
                        test_idx,
                        self.complete_path(&self.path[..test_idx], &self.path[test_idx..])?,
                    ))
                })
                .chain(std::iter::once(self.complete_path(&[], &[]).map(|res| (0, res))).flatten());

            if let Some((trunc_len, new_path)) = new_paths.next() {
                let mut new_path: Vec<_> = new_path.collect();
                new_path.reverse();

                self.path.truncate(trunc_len);
                self.path.append(&mut new_path);

                Some(true)
            } else {
                Some(false)
            }
        } else {
            Some(true)
        }
    }

    /// Returns the append to the path, backwards, if a valid one exists.
    fn complete_pathcur_pos
        &self,
        truncated_path: &[Pos2D],
        failed_prior: &[Pos2D],
    ) -> Option<RevLinkedNodeIter> {
        let final_pos = Pos2D::new(self.grid.x_max() - 1, self.grid.y_max() - 1);

        let mut visited = HashSet::new();
        visited.extend(truncated_path.iter());
        visited.extend(failed_prior);

        let mut to_visit = BinaryHeap::from([Reverse(MemorySteppedPos {
            pos: Pos2D::new(0, 0),
            steps: 0,
            memory: RevLinkedNodeInternal::new(),
        })]);

        while to_visit.peek().map(|x| x.0.pos != final_pos) == Some(true) {
            let top = to_visit
                .pop()
                .expect("at least one element by prior check")
                .0;
            if visited.contains(&top.pos) {
                continue;
            }
            visited.insert(top.pos);

            let new_memory = RevLinkedNodeInternal::push(top.memory, top.pos);

            let new_visit_locs = Direction::all()
                .into_iter()
                .flat_map(|dir| top.pos.step_dir(dir))
                .filter(|new_pos| !visited.contains(new_pos))
                .filter(|new_pos| {
                    if let Some(blocked) = self.grid.get(*new_pos) {
                        !*blocked
                    } else {
                        false
                    }
                })
                .map(|new_pos| {
                    Reverse(MemorySteppedPos {
                        pos: new_pos,
                        steps: top.steps + 1,
                        memory: new_memory.clone(),
                    })
                });
            to_visit.extend(new_visit_locs);
        }

        to_visit
            .peek()
            .map(|x| RevLinkedNodeInternal::iter(x.0.memory.clone()))
    }
}
