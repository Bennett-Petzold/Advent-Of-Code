use advent_rust_lib::read::input;

fn rotations() -> impl Iterator<Item = i64> {
    input().map(|line| {
        let num: i64 = str::parse(&line[1..]).unwrap();
        match line.chars().next().unwrap() {
            'R' => num,
            'L' => -num,
            x => panic!("Input WTF: {x}"),
        }
    })
}

fn part1<I: IntoIterator<Item = i64>>(rotations: I) -> u64 {
    let mut pos = 50;
    let mut zeroes = 0;

    for rot in rotations {
        pos = (pos + rot) % 100;
        // Not really necessary but I like the branchless trick
        // Rust bools are always 1 if true.
        zeroes += (pos == 0) as u64;
    }

    zeroes
}

fn part2<I: IntoIterator<Item = i64>>(rotations: I) -> i64 {
    let mut pos = 50;
    let mut zeroes = 0;

    for rot in rotations {
        let new_value = pos + rot;
        // Signum used to capture new_value == 0 as a switch
        let switched_direction = (pos != 0) && (new_value.signum() != pos.signum());

        // Captures forward zero crosses
        zeroes += (new_value).abs() / 100;
        // Captures a backwards zero cross
        // Remaining backwards motion is equivalent to forward
        zeroes += (switched_direction) as i64;

        pos = new_value % 100;
    }

    zeroes
}

// Around 1.2 ms on my system with my input.
fn main() {
    let rotations: Vec<_> = rotations().collect();
    println!("Part 1: {}", part1(rotations.iter().cloned()));
    println!("Part 2: {}", part2(rotations));
}
