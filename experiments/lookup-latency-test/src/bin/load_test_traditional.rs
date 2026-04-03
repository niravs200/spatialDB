use serde_json::json;
use lookup_latency_test::lookup_traditional::TraditionalLookUp;

const TOTAL: usize = 10_000_000;
const SHARDS: usize = 5;

fn main() {
    println!("=== Traditional Lookup Load Test ===");
    println!("Inserting {} records across {} shards...", TOTAL, SHARDS);

    let lookup = TraditionalLookUp::new(SHARDS);
    let shard_ids: Vec<String> = lookup.shard_ids();

    let start = std::time::Instant::now();

    for i in 0..TOTAL {
        let id = format!("record-{}", i);
        let data = json!({ "health": 100 });
        let shard_id = &shard_ids[i % shard_ids.len()];
        lookup.insert_record(shard_id, &id, data);
    }

    let elapsed = start.elapsed();

    // Sanity check
    let exists = lookup.get_record("record-99").is_some();

    println!("Record 'record-99' exists: {}", exists);
    println!("Total time: {:.2?}", elapsed);
    println!(
        "Throughput: {:.0} inserts/sec",
        TOTAL as f64 / elapsed.as_secs_f64()
    );
}
