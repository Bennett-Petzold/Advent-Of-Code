#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DirectionSet {
    Empty,
    One([Direction; 1]),
    Two([Direction; 2]),
    Three([Direction; 3]),
    Four,
}

impl Iterator for DirectionSet {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DirectionSet::Empty => None,
            DirectionSet::One(dir) => {
                let ret = Some(dir[0]);
                *self = DirectionSet::Empty;
                ret
            }
            DirectionSet::Two(dirs) => {
                let ret = Some(dirs[1]);
                *self = DirectionSet::One([dirs[0]]);
                ret
            }
            DirectionSet::Three(dirs) => {
                let ret = Some(dirs[2]);
                *self = DirectionSet::Two([dirs[0], dirs[1]]);
                ret
            }
            DirectionSet::Four => {
                let dirs = Direction::all();
                let ret = Some(dirs[3]);
                *self = DirectionSet::Three([dirs[0], dirs[1], dirs[2]]);
                ret
            }
        }
    }
}

impl DirectionSet {
    pub fn reverse(self) -> Self {
        match self {
            DirectionSet::Empty => DirectionSet::Four,
            DirectionSet::One([x]) => DirectionSet::Three(match x {
                Direction::Up => [Direction::Down, Direction::Left, Direction::Right],
                Direction::Down => [Direction::Up, Direction::Left, Direction::Right],
                Direction::Left => [Direction::Up, Direction::Down, Direction::Right],
                Direction::Right => [Direction::Up, Direction::Down, Direction::Left],
            }),
            DirectionSet::Two([x, y]) => DirectionSet::Two([x.reverse(), y.reverse()]),
            DirectionSet::Three(vals) => DirectionSet::One([Direction::all()
                .into_iter()
                .find(|dir| vals.contains(&dir))
                .expect("There is always one remaining direction")]),
            DirectionSet::Four => DirectionSet::Empty,
        }
    }

    pub fn all() -> Self {
        DirectionSet::Four
    }

    pub fn single(dir: Direction) -> Self {
        DirectionSet::One([dir])
    }
}

impl Add<Direction> for DirectionSet {
    type Output = Self;

    fn add(self, rhs: Direction) -> Self::Output {
        if self == DirectionSet::Four || self.into_iter().any(|x| x == rhs) {
            self
        } else {
            match self {
                DirectionSet::Empty => DirectionSet::One([rhs]),
                DirectionSet::One([x]) => DirectionSet::Two([x, rhs]),
                DirectionSet::Two([x, y]) => DirectionSet::Three([x, y, rhs]),
                DirectionSet::Three(_) => DirectionSet::Four,
                // Covered by earlier check
                DirectionSet::Four => unreachable!(),
            }
        }
    }
}

impl Add for DirectionSet {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match rhs {
            DirectionSet::Empty => self,
            DirectionSet::One([rhs_val]) => self + rhs_val,
            x => x.into_iter().fold(self, |acc, dir| acc + dir),
        }
    }
}

impl Trail {
    fn surrounding_valid_with_dirs(
        &self,
        point: Pos2D,
        height: u8,
        check_directions: DirectionSet,
    ) -> impl Iterator<Item = (Pos2D, DirectionSet)> + use<'_> {
        check_directions
            .flat_map(move |dir| {
                point
                    .step_dir(dir)
                    .map(|next_point| (next_point, DirectionSet::single(dir)))
            })
            .filter(move |(next_point, _)| {
                self.arr[self.flatten_point(*next_point)].height == height
            })
    }

    pub fn num_paths_with_dirs(&mut self) -> u64 {
        let mut endpoints: Vec<_> = self
            .arr
            .iter()
            .enumerate()
            .filter(|(_, loc)| (loc.height == 9))
            .map(|(idx, _)| (ValuedLoc::from_nine(self, idx), DirectionSet::all()))
            .collect();

        for height in (0..9).rev() {
            let mut new_endpoints: Vec<(ValuedLoc, DirectionSet)> =
                Vec::with_capacity(endpoints.len() * 4);

            let new_endpoint_iter =
                std::mem::take(&mut endpoints)
                    .into_iter()
                    .flat_map(|(loc, dirs)| {
                        self.surrounding_valid_with_dirs(loc.pos, height, dirs).map(
                            move |(next_point, dirs)| {
                                (
                                    ValuedLoc {
                                        pos: next_point,
                                        count: loc.count,
                                    },
                                    dirs,
                                )
                            },
                        )
                    });

            for endpoint in new_endpoint_iter {
                new_endpoints.binary_search_by_key(&endpoint.0.pos, |x| x.0.pos);
            }

            endpoints = new_endpoints;
        }
        todo!()
    }
}
