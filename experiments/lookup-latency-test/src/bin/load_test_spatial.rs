use serde_json::json;
use lookup_latency_test::lookup_spatial::SpatialLookup;

const TOTAL: u64 = 10_000_000;
const SHARDS: u64 = 5;

fn main() {
    println!("=== Spatial Lookup Load Test ===");
    println!("Inserting {} records across {} shards...", TOTAL, SHARDS);

    let lookup = SpatialLookup::new(SHARDS, TOTAL);

    let start = std::time::Instant::now();

    for i in 0..TOTAL {
        let data = json!({ "health": 100 });
        lookup.insert_record(i, data);
    }

    let elapsed = start.elapsed();

    // Sanity check
    let exists = lookup
        .get_shard(99)
        .and_then(|shard| shard.get(&99u64.to_string()))
        .is_some();

    println!("Record 99 exists: {}", exists);
    println!("Total time: {:.2?}", elapsed);
    println!(
        "Throughput: {:.0} inserts/sec",
        TOTAL as f64 / elapsed.as_secs_f64()
    );
}
