use clap::Parser;
use std::sync::{Arc, atomic::{AtomicU32, Ordering, fence}, mpsc, Barrier};
use std::thread;
use rand::Rng;

static X: AtomicU32 = AtomicU32::new(0);
static Y: AtomicU32 = AtomicU32::new(0);
static R1: AtomicU32 = AtomicU32::new(1);
static R2: AtomicU32 = AtomicU32::new(1);


#[derive(Parser, Debug)]
#[command(version = "1.0", about = "A tool to demonstrate memory ordering effects.", long_about = None)]
struct Args {
    /// Memory Ordering to use
    #[arg(short, long, default_value="Relaxed")]
    ordering: String,
    #[arg(short, long)]
    barrier: bool,
}

fn parse_ordering(ordering: &str) -> (Ordering, Ordering) {
    match ordering {
        "SeqCst" => (Ordering::SeqCst, Ordering::SeqCst),
        "AcquireRelease" => (Ordering::Acquire, Ordering::Release),
        _ => (Ordering::Relaxed, Ordering::Relaxed), // Default to Relaxed
    }
}

fn main() {

    let args = Args::parse();
    println!("args = {:#?}", args);

    let (load_ordering, store_ordering) = parse_ordering(&args.ordering);

    let barrier = Arc::new(Barrier::new(3));
    let (tx_end, rx_end) = mpsc::channel::<()>();

    let tx_end_clone = tx_end.clone();
    let barrier_clone = Arc::clone(&barrier);

    let _handle1 = thread::spawn(move || loop {
        barrier_clone.wait();

        let mut rng = rand::thread_rng();
        while rng.gen_range(0..8) != 0 {} // Random delay

	    // thread 1 Store-Load
        X.store(1, store_ordering);
        if args.barrier {
	        // apply memory barrier
            fence(Ordering::SeqCst);
        }
        R1.store(Y.load(load_ordering), Ordering::SeqCst);

        tx_end.send(()).unwrap();
    });

    let barrier_clone = Arc::clone(&barrier);

    let _handle2 = thread::spawn(move || loop {
        barrier_clone.wait();

        let mut rng = rand::thread_rng();
        while rng.gen_range(0..8) != 0 {} // Random delay

	    // thread 2 Store-Load
        Y.store(1, store_ordering);
        if args.barrier {
            fence(Ordering::SeqCst);
        }
        R2.store(X.load(load_ordering), Ordering::SeqCst);

        tx_end_clone.send(()).unwrap();
    });

    let mut reorders: u64 = 0;
    let mut iterations: u64 = 0;
    loop {
        X.store(0, Ordering::SeqCst);
        Y.store(0, Ordering::SeqCst);

        barrier.wait();

        rx_end.recv().unwrap();
        rx_end.recv().unwrap();

        let r1 = R1.load(Ordering::SeqCst);
        let r2 = R2.load(Ordering::SeqCst);

        if r1 == 0 && r2 == 0 {
            reorders += 1;
            println!("{} Reorders observed after {} iterations", reorders, iterations);
        }

        iterations += 1;
    }
}
