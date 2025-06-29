use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::io::{Write};
use crdts::bft_crdts::bft_crdt::{BFTCRDTHandler};
use crdts::bft_crdts::bft_orset::{BFTORSet};
use crdts::crdts::orset::{ORSet};
use crate::common::get_output_file;

/// Removes the top and bottom 5% of measurements
fn trim_outliers(times: &[Duration]) -> Vec<Duration> {
    if times.len() <= 2 {
        return times.to_vec();
    }
    
    let mut sorted = times.to_vec();
    sorted.sort();
    
    let trim_count = (times.len() as f64 * 0.05).ceil() as usize;
    let end = times.len() - trim_count;
    
    sorted[trim_count..end].to_vec()
}

pub fn orset_add1() {
    let num_runs = 100;
    let num_ops = 5000;
    let mut out = get_output_file("orset_add1");

    writeln!(out, "Comparing ORSet and BFTOrSet Add operation performance...").unwrap();
    writeln!(out, "{} runs", num_runs).unwrap();

    let mut orset_times: Vec<Duration> = Vec::with_capacity(num_runs);
    let mut bft_orset_times: Vec<Duration> = Vec::with_capacity(num_runs);

    // ORSet
    for run in 0..num_runs {
        let mut orset = ORSet::new();
        let start = Instant::now();
        for i in 0..num_ops {
            let random_content: String = (0..10)
                .map(|_| rand::random::<u8>() as char)
                .collect();
            orset.add(random_content, i as u64);
        }
        let elapsed = start.elapsed();
        orset_times.push(elapsed / num_ops as u32);
        writeln!(out, "ORSet {:?}", elapsed / num_ops as u32).unwrap();
    }

    // BFTORSet
    for run in 0..num_runs {
        let bft_orset = BFTORSet::new();
        let mut handler = BFTCRDTHandler::new(bft_orset);
        let start = Instant::now();
        for i in 0..num_ops {
            let random_content: String = (0..10)
                .map(|_| rand::random::<u8>() as char)
                .collect();
            let addop = handler.crdt.add(random_content);
            handler.handle_local_op(addop);
        }
        let elapsed = start.elapsed();
        bft_orset_times.push(elapsed / num_ops as u32);
        writeln!(out, "BFTOrSet {:?}", elapsed / num_ops as u32).unwrap();
    }

    fn stats(times: &[Duration]) -> (Duration, Duration, Duration) {
        let trimmed = trim_outliers(times);
        let min = *trimmed.iter().min().unwrap();
        let max = *trimmed.iter().max().unwrap();
        let sum: Duration = trimmed.iter().sum();
        let avg = sum / (trimmed.len() as u32);
        (avg, min, max)
    }

    let (orset_avg, orset_min, orset_max) = stats(&orset_times);
    let (bft_avg, bft_min, bft_max) = stats(&bft_orset_times);

    writeln!(out, "\n=== ORSet Add Performance (5% trimmed) ===").unwrap();
    writeln!(out, "Average: {:?}", orset_avg).unwrap();
    writeln!(out, "Min:     {:?}", orset_min).unwrap();
    writeln!(out, "Max:     {:?}", orset_max).unwrap();

    writeln!(out, "\n=== BFTORSet Add Performance (5% trimmed) ===").unwrap();
    writeln!(out, "Average: {:?}", bft_avg).unwrap();
    writeln!(out, "Min:     {:?}", bft_min).unwrap();
    writeln!(out, "Max:     {:?}", bft_max).unwrap();
}

pub fn orset_remove1() {
    let num_ops = 5000;
    let num_runs = 100;
    let mut out = get_output_file("orset_remove1");

    writeln!(out, "\nComparing ORSet and BFTOrSet Remove operation performance...").unwrap();
    writeln!(out, "Each run: {} Remove ops, {} runs", num_ops, num_runs).unwrap();

    let mut orset_times: Vec<Duration> = Vec::with_capacity(num_runs);
    let mut bft_orset_times: Vec<Duration> = Vec::with_capacity(num_runs);

    // ORSet
    for run in 0..num_runs {
        let mut orset = ORSet::new();
        let mut id2content = HashMap::new();
        // Pre-populate
        for i in 0..num_ops {
            let random_content: String = (0..10)
                .map(|_| rand::random::<u8>() as char)
                .collect();
            orset.add(random_content.clone(), i);
            id2content.insert(i, random_content);
        }
        let start = Instant::now();
        for i in 0..num_ops {
            orset.remove(id2content[&i].clone(), vec![i]);
        }
        let elapsed = start.elapsed();
        orset_times.push(elapsed / num_ops as u32);
        writeln!(out, "ORSet {:?}", elapsed / num_ops as u32).unwrap();
    }

    // BFTOrSet
    for run in 0..num_runs {
        let mut bft_orset = BFTORSet::new();
        let mut handler = BFTCRDTHandler::new(bft_orset);
        let mut id2content = HashMap::new();
        // Pre-populate
        for i in 0..num_ops {
            let random_content: String = (0..10)
                .map(|_| rand::random::<u8>() as char)
                .collect();
            let addop = handler.crdt.add(random_content.clone());
            handler.handle_local_op(addop);
            id2content.insert(i, random_content);
        }
        let mut ids = vec![];
        for i in 0..num_ops {
            let orset_id = handler.crdt.get_ids(id2content[&i].clone());
            // convert from hashset to vec
            let mut ids_vec = vec![];
            for id in orset_id {
                ids_vec.push(id);
            }
            ids.push(ids_vec);
        }
        let start = Instant::now();
        for i in 0..num_ops {
            let removeop = handler.crdt.remove(id2content[&i].clone(), ids[i].clone());
            handler.handle_local_op(removeop);
        }
        let elapsed = start.elapsed();
        bft_orset_times.push(elapsed / num_ops as u32);
        writeln!(out, "BFTOrSet {:?}", elapsed / num_ops as u32).unwrap();
    }

    fn stats(times: &[Duration]) -> (Duration, Duration, Duration) {
        let trimmed = trim_outliers(times);
        let min = *trimmed.iter().min().unwrap();
        let max = *trimmed.iter().max().unwrap();
        let sum: Duration = trimmed.iter().sum();
        let avg = sum / (trimmed.len() as u32);
        (avg, min, max)
    }

    let (orset_avg, orset_min, orset_max) = stats(&orset_times);
    let (bft_avg, bft_min, bft_max) = stats(&bft_orset_times);

    writeln!(out, "\n=== ORSet Remove Performance (5% trimmed) ===").unwrap();
    writeln!(out, "Average: {:?}", orset_avg).unwrap();
    writeln!(out, "Min:     {:?}", orset_min).unwrap();
    writeln!(out, "Max:     {:?}", orset_max).unwrap();

    writeln!(out, "\n=== BFTOrSet Remove Performance (5% trimmed) ===").unwrap();
    writeln!(out, "Average: {:?}", bft_avg).unwrap();
    writeln!(out, "Min:     {:?}", bft_min).unwrap();
    writeln!(out, "Max:     {:?}", bft_max).unwrap();
}
