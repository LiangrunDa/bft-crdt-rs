use std::time::{Duration, Instant};
use std::io::{Write};
use crdts::bft_crdts::bft_crdt::{BFTCRDTHandler};
use crdts::bft_crdts::bft_rga::{BFTRGA};
use crdts::crdts::crdt::CRDT;
use crdts::crdts::rga::{RGA};
use crate::common::get_output_file;
use rand;

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

pub fn rga_insert1() {
    let num_runs = 100;
    let num_ops = 5000;
    let mut out = get_output_file("rga_insert1");

    writeln!(out, "Comparing RGA and BFTRGA Insert operation performance...").unwrap();
    writeln!(out, "{} runs", num_runs).unwrap();

    let mut rga_times: Vec<Duration> = Vec::with_capacity(num_runs);
    let mut bft_rga_times: Vec<Duration> = Vec::with_capacity(num_runs);

    // RGA
    for run in 0..num_runs {
        let mut rga = RGA::new();
        let start = Instant::now();
        for i in 0..num_ops {
            let random_content: String = (0..10)
                .map(|_| rand::random::<u8>() as char)
                .collect();
            let insert_op = rga.insert(i, random_content, i as u64).unwrap();
            rga.interpret_op(&insert_op);
        }
        let elapsed = start.elapsed();
        rga_times.push(elapsed / num_ops as u32);
        writeln!(out, "RGA {:?}", elapsed / num_ops as u32).unwrap();
    }

    // BFTRGA
    for run in 0..num_runs {
        let mut bft_rga = BFTRGA::new();
        let mut handler = BFTCRDTHandler::new(bft_rga);
        let start = Instant::now();
        for i in 0..num_ops {
            let random_content: String = (0..10)
                .map(|_| rand::random::<u8>() as char)
                .collect();
            let insert_op = handler.crdt.insert(i, random_content, format!("id_{}", i)).unwrap();
            handler.handle_local_op(insert_op);
        }
        let elapsed = start.elapsed();
        bft_rga_times.push(elapsed / num_ops as u32);
        writeln!(out, "BFTRGA {:?}", elapsed / num_ops as u32).unwrap();
    }

    fn stats(times: &[Duration]) -> (Duration, Duration, Duration) {
        let trimmed = trim_outliers(times);
        let min = *trimmed.iter().min().unwrap();
        let max = *trimmed.iter().max().unwrap();
        let sum: Duration = trimmed.iter().sum();
        let avg = sum / (trimmed.len() as u32);
        (avg, min, max)
    }

    let (rga_avg, rga_min, rga_max) = stats(&rga_times);
    let (bft_avg, bft_min, bft_max) = stats(&bft_rga_times);

    writeln!(out, "\n=== RGA Insert Performance (5% trimmed) ===").unwrap();
    writeln!(out, "Average: {:?}", rga_avg).unwrap();
    writeln!(out, "Min:     {:?}", rga_min).unwrap();
    writeln!(out, "Max:     {:?}", rga_max).unwrap();

    writeln!(out, "\n=== BFTRGA Insert Performance (5% trimmed) ===").unwrap();
    writeln!(out, "Average: {:?}", bft_avg).unwrap();
    writeln!(out, "Min:     {:?}", bft_min).unwrap();
    writeln!(out, "Max:     {:?}", bft_max).unwrap();
}

pub fn rga_delete1() {
    let num_ops = 5000;
    let num_runs = 100;
    let mut out = get_output_file("rga_delete1");

    writeln!(out, "\nComparing RGA and BFTRGA Delete operation performance...").unwrap();
    writeln!(out, "Each run: {} Delete ops, {} runs", num_ops, num_runs).unwrap();

    let mut rga_times: Vec<Duration> = Vec::with_capacity(num_runs);
    let mut bft_rga_times: Vec<Duration> = Vec::with_capacity(num_runs);

    // RGA
    for run in 0..num_runs {
        let mut rga = RGA::new();
        // Pre-populate
        for i in 0..num_ops {
            let random_content: String = (0..10)
                .map(|_| rand::random::<u8>() as char)
                .collect();
            let insert_op = rga.insert(i, random_content, i as u64).unwrap();
            rga.interpret_op(&insert_op);
        }
        let start = Instant::now();
        for i in 0..num_ops {
            let delete_op = rga.delete(0).unwrap();
            rga.interpret_op(&delete_op);
        }
        let elapsed = start.elapsed();
        rga_times.push(elapsed / num_ops as u32);
        writeln!(out, "RGA {:?}", elapsed / num_ops as u32).unwrap();
    }

    // BFTRGA
    for run in 0..num_runs {
        let mut bft_rga = BFTRGA::new();
        let mut handler = BFTCRDTHandler::new(bft_rga);
        // Pre-populate
        for i in 0..num_ops {
            let random_content: String = (0..10)
                .map(|_| rand::random::<u8>() as char)
                .collect();
            let insert_op = handler.crdt.insert(i, random_content, format!("id_{}", i)).unwrap();
            handler.handle_local_op(insert_op);
        }
        let start = Instant::now();
        for i in 0..num_ops {
            let delete_op = handler.crdt.delete(0).unwrap();
            handler.handle_local_op(delete_op);
        }
        let elapsed = start.elapsed();
        bft_rga_times.push(elapsed / num_ops as u32);
        writeln!(out, "BFTRGA {:?}", elapsed / num_ops as u32).unwrap();
    }

    fn stats(times: &[Duration]) -> (Duration, Duration, Duration) {
        let trimmed = trim_outliers(times);
        let min = *trimmed.iter().min().unwrap();
        let max = *trimmed.iter().max().unwrap();
        let sum: Duration = trimmed.iter().sum();
        let avg = sum / (trimmed.len() as u32);
        (avg, min, max)
    }

    let (rga_avg, rga_min, rga_max) = stats(&rga_times);
    let (bft_avg, bft_min, bft_max) = stats(&bft_rga_times);

    writeln!(out, "\n=== RGA Delete Performance (5% trimmed) ===").unwrap();
    writeln!(out, "Average: {:?}", rga_avg).unwrap();
    writeln!(out, "Min:     {:?}", rga_min).unwrap();
    writeln!(out, "Max:     {:?}", rga_max).unwrap();

    writeln!(out, "\n=== BFTRGA Delete Performance (5% trimmed) ===").unwrap();
    writeln!(out, "Average: {:?}", bft_avg).unwrap();
    writeln!(out, "Min:     {:?}", bft_min).unwrap();
    writeln!(out, "Max:     {:?}", bft_max).unwrap();
}
