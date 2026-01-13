# Execution scheduler benchmark (MockExecutionTarget)

本报告使用 `backend/src/bin/compare_execution_modes.rs`（MockExecutionTarget + 固定 sleep）生成，可复现、避免真实 LLM 带来的波动。

## Commands

```bash
cd backend

# 调度开销基准（NFR1/NFR4）
cargo run --bin compare_execution_modes -- --benchmark --batch-size 64 --sleep-ms 20
cargo run --bin compare_execution_modes -- --benchmark --batch-size 256 --sleep-ms 20

# 串行 vs 并行质量漂移（NFR22 / AC4）
cargo run --bin compare_execution_modes -- --batch-size 256 --max-concurrency 8 --sleep-ms 20
```

## NFR Validation Summary

| Item | Requirement | Evidence | Status |
|---|---|---|---|
| NFR1 | scheduling_overhead_ms < 100ms (exclude model time) | 见下文 `batch_size=64/256` 的最大开销 | ✅ |
| NFR4 | 并行接近线性加速 | wall_clock_ms 随 N 近似按 `1/N` 收敛 | ✅ |
| NFR22 / AC4 | serial vs parallel 差异 < 5%（passed/mean_score） | `abs_delta_*` | ✅ |

## Benchmark: batch_size=64, sleep_ms_per_call=20

- baseline_serial_wall_clock_ms: 1437

| max_concurrency | wall_clock_ms | expected_ms | scheduling_overhead_ms |
|---:|---:|---:|---:|
| 1 | 1435 | 1437 | 0 |
| 2 | 718 | 718 | 0 |
| 4 | 363 | 359 | 4 |
| 8 | 176 | 179 | 0 |

## Benchmark: batch_size=256, sleep_ms_per_call=20

- baseline_serial_wall_clock_ms: 5769

| max_concurrency | wall_clock_ms | expected_ms | scheduling_overhead_ms |
|---:|---:|---:|---:|
| 1 | 5772 | 5769 | 3 |
| 2 | 2887 | 2884 | 3 |
| 4 | 1451 | 1442 | 9 |
| 8 | 735 | 721 | 14 |

## Quality Drift Check: serial vs parallel (batch_size=256, max_concurrency=8)

| mode | passed_rate | mean_score |
|---|---:|---:|
| serial | 100.00% | 1.0000 |
| parallel | 100.00% | 1.0000 |

- abs_delta_passed_rate: 0.00 percentage points
- abs_delta_mean_score: 0.0000
