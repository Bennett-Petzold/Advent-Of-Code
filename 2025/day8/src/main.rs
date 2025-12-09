use std::{
    array,
    collections::{HashMap, HashSet},
    env::args,
};

use advent_rust_lib::{posn::Pos, read::filtered_input};

type Pos3D = Pos<u64, 3>;

fn connect_cable(
    chains: &mut HashMap<Pos3D, usize>,
    active_idx: &mut HashSet<usize>,
    new_chain_idx: &mut usize,
    pos1: Pos3D,
    pos2: Pos3D,
) {
    match (chains.get(&pos1).copied(), chains.get(&pos2).copied()) {
        // Reject duplicates
        (Some(l_idx), Some(r_idx)) => {
            if l_idx != r_idx {
                // Map the chains together
                chains
                    .iter_mut()
                    .filter(|(_, idx)| **idx == l_idx)
                    .for_each(|(_, idx)| *idx = r_idx);
                // Index is overwritten
                let _ = active_idx.remove(&l_idx);
            }
        }
        (None, Some(r_idx)) => {
            chains.insert(pos1, r_idx);
        }
        (Some(l_idx), None) => {
            chains.insert(pos2, l_idx);
        }
        (None, None) => {
            chains.insert(pos1, *new_chain_idx);
            chains.insert(pos2, *new_chain_idx);
            active_idx.insert(*new_chain_idx);
            *new_chain_idx += 1;
        }
    }
}

/// Takes about 164 ms on my machine.
fn main() {
    let boxes: Vec<_> = filtered_input(&[1])
        .map(|line| {
            let mut parts = line.split(',').map(|num| num.parse().unwrap());
            Pos3D::new(array::from_fn(|_| parts.by_ref().next().unwrap()))
        })
        .collect();

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
    let mut distances_iter = distances.into_iter();

    let mut chains: HashMap<Pos3D, usize> = HashMap::new();
    let mut active_idx: HashSet<usize> = HashSet::new();

    let mut new_chain_idx = 0;

    // ----- Part 1 ----- //
    let reps = args().nth(2).unwrap().parse().unwrap();
    for (_, pos1, pos2) in distances_iter.by_ref().take(reps) {
        connect_cable(&mut chains, &mut active_idx, &mut new_chain_idx, pos1, pos2);
    }

    let mut lengths = vec![0; new_chain_idx];
    for idx in chains.values() {
        lengths[*idx] += 1;
    }
    lengths.sort_unstable();

    println!("Part 1: {}", lengths.iter().rev().take(3).product::<u64>());

    // ----- Part 2 ----- //
    for (_, pos1, pos2) in distances_iter {
        connect_cable(&mut chains, &mut active_idx, &mut new_chain_idx, pos1, pos2);

        if (chains.len() == boxes.len()) && (active_idx.len() == 1) {
            println!("Part 2: {}", pos1.coordinates[0] * pos2.coordinates[0]);
            break;
        }
    }
}
