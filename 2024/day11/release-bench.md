| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/main input 75` | 27.6 ± 0.7 | 27.4 | 34.5 | 3.30 ± 0.20 |
| `target/release/vec_reuse input 75` | 21.2 ± 0.1 | 21.1 | 21.5 | 2.54 ± 0.14 |
| `target/release/vec_combined_remerge input 75` | 8.4 ± 0.5 | 8.0 | 11.7 | 1.00 |
| `target/release/special_case input 75` | 9.1 ± 1.7 | 8.0 | 15.3 | 1.09 ± 0.22 |
| `target/release/cache_condition input 75` | 9.4 ± 1.3 | 7.9 | 12.6 | 1.13 ± 0.17 |
| `target/release/math_trickery input 75` | 12.7 ± 1.3 | 12.0 | 15.9 | 1.52 ± 0.17 |
| `target/release/space_savings input 75` | 11.7 ± 0.3 | 11.6 | 13.4 | 1.40 ± 0.09 |
| `target/release/space_savings_1 input 75` | 12.0 ± 0.6 | 11.5 | 13.8 | 1.43 ± 0.11 |
| `target/release/skip_last_sort input 75` | 8.4 ± 0.7 | 7.7 | 11.3 | 1.00 ± 0.10 |
| `target/release/space_savings_skip_last_sort input 75` | 11.8 ± 0.8 | 11.3 | 14.9 | 1.41 ± 0.13 |
| `target/release/cache_logs input 75` | 9.2 ± 0.3 | 9.1 | 12.3 | 1.11 ± 0.07 |
| `target/release/full_cache input 75` | 12.3 ± 0.3 | 12.0 | 15.6 | 1.47 ± 0.09 |
| `target/release/comp_cache_only input 75` | 10.9 ± 1.1 | 10.3 | 14.3 | 1.31 ± 0.15 |
| `target/release/keep_working_vec input 75` | 9.4 ± 2.2 | 7.7 | 14.8 | 1.12 ± 0.27 |
