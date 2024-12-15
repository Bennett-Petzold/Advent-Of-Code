use advent_rust_lib::{
    direction::Direction,
    grid::{GridEntry, Pos2D, RectangleGrid},
    iter::ArrayIter,
    read::input,
};

fn main() {
    let mut input = input();

    let input_to_blank = input.by_ref().take_while(|line| !line.trim().is_empty());
    let map = Map::from_iter(input_to_blank).unwrap();

    let directions: Vec<_> = input
        .flat_map(|line| line.chars().flat_map(parse_dir_arrow).collect::<Vec<_>>())
        .collect();

    part_1(map.clone(), &directions);
    part_2(map, &directions);
}

fn part_1(mut map: Map, directions: &[Direction]) {
    for dir in directions {
        map.step(*dir);
    }

    #[cfg(feature = "print")]
    {
        map.print();
        println!()
    }

    let gps_coord_sum: usize = map
        .positioned_items()
        .filter(|entry| *entry.value == Some(Element::Box))
        .map(|entry| {
            let pos = entry.position();
            (pos.y * 100) + pos.x
        })
        .sum();

    println!("{gps_coord_sum}");
}

fn part_2(map: Map, directions: &[Direction]) {
    let mut map = WideMap::from_regular(map);

    for dir in directions {
        map.step(*dir);

        #[cfg(debug_assertions)]
        if !map.validate_boxes() {
            #[cfg(feature = "print")]
            {
                map.print();
                println!()
            }
            panic!("{}", dir);
        }
    }

    #[cfg(feature = "print")]
    {
        map.print();
        println!()
    }

    let gps_coord_sum: usize = map
        .positioned_items()
        .filter(|entry| *entry.value == Some(WideElement::LeftBox))
        .map(|entry| {
            let pos = entry.position();
            (pos.y * 100) + pos.x
        })
        .sum();

    println!("{gps_coord_sum}");
}

// -------------------------------------------------- //

#[derive(Debug, Clone)]
struct Map {
    grid: RectangleGrid<Option<Element>>,
    robot: Pos2D,
}

impl Map {
    pub fn from_iter<S, I>(iter: I) -> Option<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut robot = None;

        let iter = iter
            .into_iter()
            .enumerate()
            .inspect(|(y, line)| {
                if let Some(x) = line.as_ref().find('@') {
                    robot = Some(Pos2D::new(x, *y));
                }
            })
            .map(|(_, line)| {
                line.as_ref()
                    .chars()
                    .map(|c| match c {
                        '#' => Some(Element::Wall),
                        'O' => Some(Element::Box),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            });

        let grid = RectangleGrid::try_from_iter(iter).ok()?;

        Some(Self {
            grid,
            robot: robot?,
        })
    }

    pub fn step(&mut self, dir: Direction) {
        if let Some(new_robot) = self.robot.step_dir(dir) {
            if let Some(grid_entry) = self.grid.get(new_robot).and_then(|entry| *entry) {
                if grid_entry == Element::Box && self.push_box(dir, new_robot) {
                    // Box was pushed, robot can move into empty space
                    self.robot = new_robot;
                }
            } else {
                // No obstacles, robot can move
                self.robot = new_robot;
            }
        }
    }

    /// Pushes the box at `box_pos` in `dir`, if possible.
    ///
    /// Returns true if successful, false otherwise.
    fn push_box(&mut self, dir: Direction, box_pos: Pos2D) -> bool {
        if let Some(new_box) = box_pos.step_dir(dir) {
            if let Some(grid_entry) = self.grid.get(new_box).copied() {
                let success = match grid_entry {
                    Some(Element::Wall) => false,
                    None => true,
                    Some(Element::Box) => self.push_box(dir, new_box),
                };

                if success {
                    // Shift this box to its new position
                    *self
                        .grid
                        .get_mut(new_box)
                        .expect("already confirmed this location exists") = Some(Element::Box);
                    *self
                        .grid
                        .get_mut(box_pos)
                        .expect("already confirmed this location exists") = None;
                }

                success
            } else {
                // Outside of grid
                false
            }
        } else {
            // Outside of grid
            false
        }
    }

    pub fn positioned_items(&self) -> impl ArrayIter<GridEntry<Option<Element>>> {
        self.grid.positioned_items()
    }

    #[cfg(feature = "print")]
    pub fn print(&self) {
        use std::io::stdout;

        self.grid
            .print(&mut stdout(), |item| match item {
                None => " ",
                Some(Element::Wall) => "#",
                Some(Element::Box) => "O",
            })
            .unwrap();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Element {
    Wall,
    Box,
}

fn parse_dir_arrow(c: char) -> Option<Direction> {
    match c {
        '^' => Some(Direction::Up),
        'v' => Some(Direction::Down),
        '<' => Some(Direction::Left),
        '>' => Some(Direction::Right),
        _ => None,
    }
}

#[derive(Debug, Clone)]
struct WideMap {
    grid: RectangleGrid<Option<WideElement>>,
    robot: Pos2D,
}

impl WideMap {
    pub fn from_regular(map: Map) -> Self {
        let grid_iter = map.grid.lines().map(|line| {
            line.iter().flat_map(|elem| match elem {
                None => [None, None],
                Some(Element::Wall) => [Some(WideElement::Wall), Some(WideElement::Wall)],
                Some(Element::Box) => [Some(WideElement::LeftBox), Some(WideElement::RightBox)],
            })
        });
        let grid =
            RectangleGrid::try_from_iter(grid_iter).expect("All X dims are equally increased");
        let robot = Pos2D::new(map.robot.x * 2, map.robot.y);

        Self { grid, robot }
    }

    pub fn step(&mut self, dir: Direction) {
        if let Some(new_robot) = self.robot.step_dir(dir) {
            if let Some(grid_entry) = self.grid.get(new_robot).and_then(|entry| *entry) {
                if grid_entry.is_box() && self.push_box(dir, new_robot) {
                    // Box was pushed, robot can move into empty space
                    self.robot = new_robot;
                }
            } else {
                // No obstacles, robot can move
                self.robot = new_robot;
            }
        }
    }

    /// Pushes the box at `box_pos` in `dir`, if possible.
    ///
    /// Returns true if successful, false otherwise.
    fn push_box(&mut self, dir: Direction, box_pos: Pos2D) -> bool {
        match dir {
            Direction::Left | Direction::Right => {
                self.push_box_side(dir == Direction::Right, box_pos)
            }
            Direction::Up | Direction::Down => {
                if let Some(item) = self.grid.get(box_pos).copied().flatten() {
                    let pos_pair = match item {
                        WideElement::Wall => {
                            panic!("Push box should only have been called with a box!")
                        }
                        WideElement::LeftBox => {
                            let other_box = Pos2D::new(box_pos.x + 1, box_pos.y);
                            [box_pos, other_box]
                        }
                        WideElement::RightBox => {
                            let other_box = Pos2D::new(box_pos.x - 1, box_pos.y);
                            [other_box, box_pos]
                        }
                    };

                    if let Some(mut locs) = self.push_box_vert(dir, pos_pair) {
                        locs.sort_unstable_by(|lhs, rhs| lhs.y.cmp(&rhs.y).then(lhs.x.cmp(&rhs.x)));
                        locs.dedup();

                        if dir == Direction::Up {
                            // Highest y first
                            for loc in &locs {
                                let val = self
                                    .grid
                                    .get_mut(*loc)
                                    .expect("found element must exist")
                                    .take();
                                let upper = loc.up().expect("must have an upper element");
                                *self
                                    .grid
                                    .get_mut(upper)
                                    .expect("upper element must be in the grid") = val;
                            }
                        } else {
                            debug_assert_eq!(dir, Direction::Down);

                            // Lowest y first
                            for loc in locs.iter().rev() {
                                let val = self
                                    .grid
                                    .get_mut(*loc)
                                    .expect("found element must exist")
                                    .take();
                                let lower = loc.down().expect("must have an lower element");
                                *self
                                    .grid
                                    .get_mut(lower)
                                    .expect("lower element must be in the grid") = val;
                            }
                        }

                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }

    /// Pushes the box at `box_pos` to the side in `dir`, if possible.
    ///
    /// Returns Some(num) for the amount of side shift, if possible.
    fn push_box_side(&mut self, right: bool, box_pos: Pos2D) -> bool {
        let next = |pos: Pos2D| if right { pos.right() } else { pos.left() };
        let mut working_box_pos = box_pos;

        loop {
            if let Some(new_box) = next(working_box_pos) {
                if let Some(grid_entry) = self.grid.get(new_box).copied() {
                    match grid_entry {
                        Some(WideElement::Wall) => return false,
                        None => break,
                        Some(WideElement::LeftBox) | Some(WideElement::RightBox) => {
                            working_box_pos = new_box;
                        }
                    }
                } else {
                    // Outside of grid
                    return false;
                }
            } else {
                // Outside of grid
                return false;
            }
        }

        // Copy grid to the side
        let line = self
            .grid
            .lines_mut()
            .nth(box_pos.y)
            .expect("this function must have been called with a valid box location");
        if right {
            let robot_x = box_pos.x - 1;
            line.copy_within(robot_x..=working_box_pos.x, robot_x + 1);
            line[robot_x] = None;
        } else {
            let robot_x = box_pos.x + 1;
            line.copy_within(working_box_pos.x..=robot_x, working_box_pos.x - 1);
        };

        true
    }

    /// Pushes the box at `box_pos` vertically in `dir`, if possible.
    ///
    /// Returns the sequence of positions if successful.
    fn push_box_vert(&mut self, dir: Direction, box_poses: [Pos2D; 2]) -> Option<Vec<Pos2D>> {
        let mut positions = box_poses.to_vec();

        for box_pos in box_poses {
            if let Some(new_box) = box_pos.step_dir(dir) {
                if let Some(grid_entry) = self.grid.get(new_box).copied() {
                    let mut new_positions = match grid_entry {
                        Some(WideElement::Wall) => return None,
                        None => vec![],
                        Some(WideElement::LeftBox) => {
                            let alt_box = Pos2D::new(new_box.x + 1, new_box.y);
                            self.push_box_vert(dir, [new_box, alt_box])?
                        }
                        Some(WideElement::RightBox) => {
                            let alt_box = Pos2D::new(new_box.x - 1, new_box.y);
                            self.push_box_vert(dir, [alt_box, new_box])?
                        }
                    };

                    positions.append(&mut new_positions);
                } else {
                    // Outside of grid
                    return None;
                }
            } else {
                // Outside of grid
                return None;
            }
        }

        Some(positions)
    }

    pub fn positioned_items(&self) -> impl ArrayIter<GridEntry<Option<WideElement>>> {
        self.grid.positioned_items()
    }

    #[cfg(feature = "print")]
    pub fn print(&self) {
        use std::io::stdout;

        self.grid
            .print(&mut stdout(), |item| match item {
                None => " ",
                Some(WideElement::Wall) => "#",
                Some(WideElement::LeftBox) => "[",
                Some(WideElement::RightBox) => "]",
            })
            .unwrap();
    }

    #[cfg(debug_assertions)]
    pub fn validate_boxes(&self) -> bool {
        self.grid.lines().all(|line| {
            let mut line = line.iter();
            while let Some(lhs) = line.next() {
                if *lhs == Some(WideElement::LeftBox)
                    && line.next().cloned().flatten() != Some(WideElement::RightBox)
                {
                    return false;
                }

                if *lhs == Some(WideElement::RightBox) {
                    return false;
                }
            }
            true
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WideElement {
    Wall,
    LeftBox,
    RightBox,
}

impl WideElement {
    fn is_box(&self) -> bool {
        [WideElement::LeftBox, WideElement::RightBox].contains(self)
    }
}
