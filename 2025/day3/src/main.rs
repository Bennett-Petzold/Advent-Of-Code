use advent_rust_lib::read::input;

fn bank(line: &str) -> Vec<u8> {
    line.trim().chars().map(|c| (c as u8) - 0x30).collect()
}

fn part1<'a, I>(banks: I) -> u64
where
    I: IntoIterator<Item = &'a Vec<u8>>,
{
    banks
        .into_iter()
        .map(|bank| {
            let mut largest = *bank.first().unwrap();
            let mut second_largest = 0;

            for val in &bank[1..(bank.len() - 1)] {
                let val = *val;
                if val > largest {
                    largest = val;
                    second_largest = 0;
                } else if val > second_largest {
                    second_largest = val;
                }
            }

            let last = *bank.last().unwrap();
            if last > second_largest {
                second_largest = last;
            }

            (largest * 10) + second_largest
        })
        .map(u64::from)
        .sum()
}

fn part2<'a, I>(banks: I) -> u64
where
    I: IntoIterator<Item = &'a Vec<u8>>,
{
    banks
        .into_iter()
        .flat_map(|bank| {
            let mut start_idx = 0;
            (0..12).rev().map(move |end_offset| {
                let end_idx = bank.len() - end_offset;

                let mut digit = 0;
                let mut found_idx = 0;

                for (later_idx, later_digit) in bank[start_idx..end_idx].iter().enumerate() {
                    if *later_digit > digit {
                        digit = *later_digit;
                        found_idx = later_idx;
                    }
                }

                start_idx += found_idx + 1;
                (digit as u64) * 10_u64.pow(end_offset as u32)
            })
        })
        .sum()
}

// Runs in about 970 microseconds on my machine.
fn main() {
    let banks: Vec<_> = input().map(|s| bank(&s)).collect();

    println!("Part 1: {}", part1(banks.iter()));
    println!("Part 2: {}", part2(banks.iter()));
}
