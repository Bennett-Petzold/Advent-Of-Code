use std::{array, collections::HashMap, env::args};

use advent_rust_lib::{
    posn::Pos,
    read::{filtered_input, input},
};

type Pos3D = Pos<u64, 3>;

fn part1(boxes: &[Pos3D], reps: usize) -> u64 {
    if boxes.is_empty() {
        return 0;
    }

    let mut ignore_idx = 0;
    let mut distances: Vec<_> = boxes
        .iter()
        .flat_map(|next_box| {
            let closest = boxes[0..ignore_idx]
                .iter()
                .chain(boxes.iter().skip(ignore_idx + 1))
                .map(|candidate| (next_box.euclid_dist(candidate), *next_box, *candidate));
            ignore_idx += 1;
            closest
        })
        .collect();

    // Smallest distance is leftmost.
    distances.sort_unstable_by(|lhs, rhs| lhs.0.total_cmp(&rhs.0));
    // Duplicates will be next to each other with flipped positions.
    distances.dedup_by(|(_, pos_lhs, _), (_, _, pos_rhs)| pos_lhs == pos_rhs);

    let mut chains: HashMap<Pos3D, usize> = HashMap::new();

    let mut new_chain_idx = 0;

    for (_, pos1, pos2) in distances.into_iter().take(reps) {
        match (chains.get(&pos1).copied(), chains.get(&pos2).copied()) {
            // Reject duplicates
            (Some(l_idx), Some(r_idx)) => {
                if l_idx != r_idx {
                    // Map the chains together
                    chains
                        .iter_mut()
                        .filter(|(_, idx)| **idx == l_idx)
                        .for_each(|(_, idx)| *idx = r_idx);
                }
            }
            (None, Some(r_idx)) => {
                chains.insert(pos1, r_idx);
            }
            (Some(l_idx), None) => {
                chains.insert(pos2, l_idx);
            }
            (None, None) => {
                chains.insert(pos1, new_chain_idx);
                chains.insert(pos2, new_chain_idx);
                new_chain_idx += 1;
            }
        }
    }

    let mut lengths = vec![0; new_chain_idx];
    for (_, idx) in chains {
        lengths[idx] += 1;
    }
    lengths.sort_unstable();

    lengths.into_iter().rev().take(3).product()
}

fn main() {
    let boxes: Vec<_> = filtered_input(&[1])
        .map(|line| {
            let mut parts = line.split(',').map(|num| num.parse().unwrap());
            Pos3D::new(array::from_fn(|_| parts.by_ref().next().unwrap()))
        })
        .collect();

    println!(
        "Part 1: {}",
        part1(&boxes, args().nth(2).unwrap().parse().unwrap())
    );
}
