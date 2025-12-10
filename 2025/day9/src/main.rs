use advent_rust_lib::{posn::Pos, read::input};

type Pos2D = Pos<u64, 2>;

fn main() {
    let red_tiles: Vec<_> = input()
        .map(|line| {
            let (x, y) = line.split_once(',').unwrap();
            Pos2D::new([x.parse().unwrap(), y.parse().unwrap()])
        })
        .collect();

    println!(
        "Part 1: {}",
        red_tiles
            .iter()
            .flat_map(|lhs| {
                red_tiles.iter().map(move |rhs| {
                    let diff = lhs.abs_diff(rhs);
                    let fluffed = Pos2D::new(diff.coordinates.map(|dim| dim + 1));
                    fluffed.area()
                })
            })
            .max()
            .unwrap()
    );
}
