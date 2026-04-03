# Load Test Results

## Results

### Spatial Lookup (`load_test_spatial`)

| Date       | Records    | Shards | Total Time | Throughput       | Notes |
|------------|------------|--------|------------|------------------|-------|
| 2026-03-04 | 10,000,000 | 5      | 3.81s      | 2,627,378 ins/s  |       |
| 2026-03-04 | 20,000,000 | 5      | 7.85s      | 2,548,953 ins/s  |       |
| 2026-03-04 | 30,000,000 | 5      | 10.98s     | 2,733,143 ins/s  |       |
| 2026-03-04 | 40,000,000 | 5      | 16.49s     | 2,425,861 ins/s  |       |
| 2026-03-04 | 50,000,000 | 5      | 19.94s     | 2,507,484 ins/s  |       |
| 2026-03-04 | 60,000,000 | 5      | 23.28s     | 2,577,141 ins/s  |       |
| 2026-03-04 | 70,000,000 | 5      | 27.02s     | 2,590,334 ins/s  |       |

---

### Traditional Lookup (`load_test_traditional`)

| Date       | Records    | Shards | Total Time | Throughput       | Notes |
|------------|------------|--------|------------|------------------|-------|
| 2026-03-04 | 10,000,000 | 5      | 8.10s      | 1,235,081 ins/s  |       |
| 2026-03-04 | 20,000,000 | 5      | 18.34s     | 1,090,670 ins/s  |       |
| 2026-03-04 | 30,000,000 | 5      | 29.37s     | 1,021,563 ins/s  |       |
| 2026-03-04 | 40,000,000 | 5      | 40.37s     | 990,931 ins/s    |       |
| 2026-03-04 | 50,000,000 | 5      | 48.99s     | 1,020,662 ins/s  |       |
| 2026-03-04 | 60,000,000 | 5      | 65.42s     | 917,166 ins/s    |       |
| 2026-03-04 | 70,000,000 | 5      | 76.76s     | 911,889 ins/s    |       |

---

## Summary

At 70M records, **Spatial is ~2.8x faster** than Traditional in raw insert throughput.

| Metric         | Spatial               | Traditional           |
|----------------|-----------------------|-----------------------|
| Throughput     | ~2.6M ins/s           | ~0.9M ins/s           |
| Lookup method  | Index by range        | Hash map scan         |
| Memory layout  | Pre-allocated shards  | Dynamic partitions    |
