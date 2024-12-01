use lazy_static::lazy_static;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

use criterion::{criterion_group, criterion_main, Criterion};
use day9::Sequence;

#[cfg(feature = "simd")]
use criterion::BenchmarkId;

lazy_static! {
    static ref SEQUENCE_I64: Vec<Sequence<i64>> = BufReader::new(File::open("input").unwrap())
        .lines()
        .map(|line| Sequence::from_str(&line.unwrap()).unwrap())
        .collect();
}

lazy_static! {
    static ref SEQUENCE_I32: Vec<Sequence<i32>> = BufReader::new(File::open("input").unwrap())
        .lines()
        .map(|line| Sequence::from_str(&line.unwrap()).unwrap())
        .collect();
}

pub fn part1_base(c: &mut Criterion) {
    let mut group = c.benchmark_group("part1_base");
    let seq_64 = SEQUENCE_I64.clone();
    let seq_32 = SEQUENCE_I32.clone();

    group.bench_function("i64", |b| {
        b.iter(|| seq_64.iter().map(|s| s.next()).sum::<i64>())
    });

    group.bench_function("i32", |b| {
        b.iter(|| seq_32.iter().map(|s| s.next()).sum::<i32>())
    });

    group.finish();
}

pub fn part1_multithreaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("part1_multithreaded");
    let seq_64 = SEQUENCE_I64.clone();
    let seq_32 = SEQUENCE_I32.clone();

    group.bench_function("i64", |b| {
        b.iter(|| seq_64.par_iter().map(|s| s.next()).sum::<i64>())
    });

    group.bench_function("i32", |b| {
        b.iter(|| seq_32.par_iter().map(|s| s.next()).sum::<i32>())
    });

    group.finish();
}

#[cfg(feature = "simd")]
macro_rules! construct_simd_test {
    ($group: ident, $i: literal, $seq_64: ident, $seq_32: ident, $func: ident) => {
        $group.bench_with_input(BenchmarkId::new("i64", $i), &$i, |b, _| {
            b.iter(|| $seq_64.iter().map(|s| s.$func::<$i>()).sum::<i64>())
        });

        $group.bench_with_input(BenchmarkId::new("i32", $i), &$i, |b, _| {
            b.iter(|| $seq_32.iter().map(|s| s.$func::<$i>()).sum::<i32>())
        });
    };
}

#[cfg(feature = "simd")]
pub fn part1_simd(c: &mut Criterion) {
    let mut group = c.benchmark_group("part1_simd");
    let seq_64 = SEQUENCE_I64.clone();
    let seq_32 = SEQUENCE_I32.clone();

    construct_simd_test!(group, 2, seq_64, seq_32, next_simd);
    construct_simd_test!(group, 4, seq_64, seq_32, next_simd);
    construct_simd_test!(group, 8, seq_64, seq_32, next_simd);
    construct_simd_test!(group, 16, seq_64, seq_32, next_simd);
    construct_simd_test!(group, 32, seq_64, seq_32, next_simd);
    construct_simd_test!(group, 64, seq_64, seq_32, next_simd);

    group.finish();
}

#[cfg(feature = "simd")]
macro_rules! construct_simd_mt_test {
    ($group: ident, $i: literal, $seq_64: ident, $seq_32: ident, $func: ident) => {
        $group.bench_with_input(BenchmarkId::new("i64", $i), &$i, |b, _| {
            b.iter(|| $seq_64.par_iter().map(|s| s.$func::<$i>()).sum::<i64>())
        });

        $group.bench_with_input(BenchmarkId::new("i32", $i), &$i, |b, _| {
            b.iter(|| $seq_32.par_iter().map(|s| s.$func::<$i>()).sum::<i32>())
        });
    };
    ($group: ident, $i: literal, $seq_64: ident, $seq_32: ident, $func: ident, $suffix: literal) => {
        $group.bench_with_input(
            BenchmarkId::new("i64".to_owned() + $suffix, $i),
            &$i,
            |b, _| b.iter(|| $seq_64.par_iter().map(|s| s.$func::<$i>()).sum::<i64>()),
        );

        $group.bench_with_input(
            BenchmarkId::new("i32".to_owned() + $suffix, $i),
            &$i,
            |b, _| b.iter(|| $seq_32.par_iter().map(|s| s.$func::<$i>()).sum::<i32>()),
        );
    };
}

#[cfg(feature = "simd")]
pub fn part1_simd_multithreaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("part1_simd_multithreaded");
    let seq_64 = SEQUENCE_I64.clone();
    let seq_32 = SEQUENCE_I32.clone();

    construct_simd_mt_test!(group, 2, seq_64, seq_32, next_simd);
    construct_simd_mt_test!(group, 4, seq_64, seq_32, next_simd);
    construct_simd_mt_test!(group, 8, seq_64, seq_32, next_simd);
    construct_simd_mt_test!(group, 16, seq_64, seq_32, next_simd);
    construct_simd_mt_test!(group, 32, seq_64, seq_32, next_simd);
    construct_simd_mt_test!(group, 64, seq_64, seq_32, next_simd);

    group.finish();
}

#[cfg(feature = "simd")]
pub fn part1_large_inputs(c: &mut Criterion) {
    let mut group = c.benchmark_group("part1_large_inputs");
    let seq_64: Vec<_> = SEQUENCE_I64
        .iter()
        .cycle()
        .take(SEQUENCE_I64.len() * 100)
        .collect();
    let seq_32: Vec<_> = SEQUENCE_I32
        .iter()
        .cycle()
        .take(SEQUENCE_I32.len() * 100)
        .collect();

    construct_simd_test!(group, 64, seq_64, seq_32, next_simd);
    construct_simd_mt_test!(group, 64, seq_64, seq_32, next_simd, "-mt");

    group.finish();
}

pub fn part2_base(c: &mut Criterion) {
    let mut group = c.benchmark_group("part2_base");
    let seq_64 = SEQUENCE_I64.clone();
    let seq_32 = SEQUENCE_I32.clone();

    group.bench_function("i64", |b| {
        b.iter(|| seq_64.iter().map(|s| s.prev()).sum::<i64>())
    });

    group.bench_function("i32", |b| {
        b.iter(|| seq_32.iter().map(|s| s.prev()).sum::<i32>())
    });

    group.finish();
}

pub fn part2_multithreaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("part2_multithreaded");
    let seq_64 = SEQUENCE_I64.clone();
    let seq_32 = SEQUENCE_I32.clone();

    group.bench_function("i64", |b| {
        b.iter(|| seq_64.par_iter().map(|s| s.prev()).sum::<i64>())
    });

    group.bench_function("i32", |b| {
        b.iter(|| seq_32.par_iter().map(|s| s.prev()).sum::<i32>())
    });

    group.finish();
}

#[cfg(feature = "simd")]
pub fn part2_simd(c: &mut Criterion) {
    let mut group = c.benchmark_group("part2_simd");
    let seq_64 = SEQUENCE_I64.clone();
    let seq_32 = SEQUENCE_I32.clone();

    construct_simd_test!(group, 2, seq_64, seq_32, prev_simd);
    construct_simd_test!(group, 4, seq_64, seq_32, prev_simd);
    construct_simd_test!(group, 8, seq_64, seq_32, prev_simd);
    construct_simd_test!(group, 16, seq_64, seq_32, prev_simd);
    construct_simd_test!(group, 32, seq_64, seq_32, prev_simd);
    construct_simd_test!(group, 64, seq_64, seq_32, prev_simd);

    group.finish();
}

#[cfg(feature = "simd")]
pub fn part2_simd_multithreaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("part2_simd_multithreaded");
    let seq_64 = SEQUENCE_I64.clone();
    let seq_32 = SEQUENCE_I32.clone();

    construct_simd_mt_test!(group, 2, seq_64, seq_32, prev_simd);
    construct_simd_mt_test!(group, 4, seq_64, seq_32, prev_simd);
    construct_simd_mt_test!(group, 8, seq_64, seq_32, prev_simd);
    construct_simd_mt_test!(group, 16, seq_64, seq_32, prev_simd);
    construct_simd_mt_test!(group, 32, seq_64, seq_32, prev_simd);
    construct_simd_mt_test!(group, 64, seq_64, seq_32, prev_simd);

    group.finish();
}

#[cfg(feature = "simd")]
pub fn part2_large_inputs(c: &mut Criterion) {
    let mut group = c.benchmark_group("part2_large_inputs");
    let seq_64: Vec<_> = SEQUENCE_I64
        .iter()
        .cycle()
        .take(SEQUENCE_I64.len() * 100)
        .collect();
    let seq_32: Vec<_> = SEQUENCE_I32
        .iter()
        .cycle()
        .take(SEQUENCE_I32.len() * 100)
        .collect();

    construct_simd_test!(group, 64, seq_64, seq_32, prev_simd);
    construct_simd_mt_test!(group, 64, seq_64, seq_32, prev_simd, "-mt");

    group.finish();
}

criterion_group!(part1, part1_base, part1_multithreaded);
#[cfg(feature = "simd")]
criterion_group!(
    part1_ex,
    part1_simd,
    part1_simd_multithreaded,
    part1_large_inputs
);

criterion_group!(part2, part2_base, part2_multithreaded);
#[cfg(feature = "simd")]
criterion_group!(
    part2_ex,
    part2_simd,
    part2_simd_multithreaded,
    part2_large_inputs
);

#[cfg(not(feature = "simd"))]
criterion_main!(part1, part2);
#[cfg(feature = "simd")]
criterion_main!(part1, part2, part1_ex, part2_ex);
