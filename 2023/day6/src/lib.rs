use std::cmp;

use anyhow::anyhow;

#[derive(Debug, PartialEq, Eq)]
pub struct Race {
    time: u64,
    distance: u64,
}

// Must be exactly two lines
pub fn parse_races<'a, S: AsRef<str>>(input: &'a [S; 2]) -> anyhow::Result<Vec<Race>> {
    let get_numbers = |s: &'a S| -> anyhow::Result<_> {
        Ok(s.as_ref()
            .split(':')
            .nth(1)
            .ok_or(anyhow!(
                "\"{}\" does not have a second field after a colon",
                input[0].as_ref()
            ))?
            .split_whitespace()
            .map(|s| s.parse()))
    };

    let (times, distances) = (get_numbers(&input[0])?, get_numbers(&input[1])?);

    times
        .zip(distances)
        .map(|(time, distance)| {
            Ok(Race {
                time: time?,
                distance: distance?,
            })
        })
        .collect::<anyhow::Result<_>>()
}

// Must be exactly two lines
pub fn parse_single_race<'a, S: AsRef<str>>(input: &'a [S; 2]) -> anyhow::Result<Race> {
    let get_numbers = |s: &'a S| -> anyhow::Result<_> {
        Ok(s.as_ref()
            .split(':')
            .nth(1)
            .ok_or(anyhow!(
                "\"{}\" does not have a second field after a colon",
                input[0].as_ref()
            ))?
            .trim()
            .replace(' ', "")
            .parse())
    };

    let (time, distance) = (get_numbers(&input[0])?, get_numbers(&input[1])?);

    Ok(Race {
        time: time?,
        distance: distance?,
    })
}

impl Race {
    /// Edges of range that exceed target distance in time
    /// Uses f64 internally, because f32 loses too much precision for part 2
    fn outperform_edges(target: u64, time: u64) -> (u64, u64) {
        // x secs holding button
        // distance = (time - x) * x
        // edges -> (time - x) * x == target_distance
        // quadratic formula to find roots!
        // a = 1, b is -time, c is distance

        let target_float = (target as f64) + 0.01;
        let time_float = time as f64;

        let sqrt_part = f64::sqrt((time_float * time_float) - (4.0 * target_float)) / 2.0;
        let b_part = time_float / 2.0;
        let lower = cmp::max(0, f64::ceil(b_part - sqrt_part) as i32) as u64;
        let higher = cmp::min(time, f64::floor(b_part + sqrt_part) as u64);
        (lower, higher)
    }

    pub fn num_ways_to_win(&self) -> u64 {
        let (lower, higher) = Self::outperform_edges(self.distance, self.time);
        (higher - lower) + 1 // Inclusive count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        let input = ["Time:      7  15   30", "Distance:  9  40  200"];
        assert_eq!(
            parse_races(&input).unwrap()[2],
            Race {
                time: 30,
                distance: 200
            }
        );
    }

    #[test]
    fn part1_test() {
        let input = ["Time:      7  15   30", "Distance:  9  40  200"];
        assert_eq!(
            parse_races(&input)
                .unwrap()
                .into_iter()
                .map(|r| r.num_ways_to_win())
                .product::<u64>(),
            288
        );
    }

    #[test]
    fn part2_test() {
        let input = ["Time:      7  15   30", "Distance:  9  40  200"];
        assert_eq!(parse_single_race(&input).unwrap().num_ways_to_win(), 71503);
    }
}
