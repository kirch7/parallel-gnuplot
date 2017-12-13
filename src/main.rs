extern crate blockcounter;
extern crate num_cpus;
extern crate threadpool;

use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        panic!("Call with: {} datafilename gnuplotfilename", args[0]);
    }

    let datafilename = &args[1];
    let gpfilename = &args[2];

    let indexes_no = blockcounter::blank_lines(2, datafilename);
    let cpus_no = num_cpus::get();

    let pool = ThreadPool::new(cpus_no);

    let (tx, _rx) = channel();
    for index in 0..indexes_no {
        let tx = tx.clone();
        let index = index;
        let datafilename = datafilename.clone();
        let gpfilename = gpfilename.clone();
        pool.execute(move || {
            let index_str = &index.to_string();
            let err_message = "Failed to execute GNUPlot with index {}.".to_string() + index_str;
            let index_argument = "INDEX=".to_string() + index_str;
            let data_argument = "DATAFILE=\"".to_string() + &datafilename + "\"";
            let _ = Command::new("gnuplot")
                .args(&["-e", &index_argument, "-e", &data_argument, &gpfilename])
                .output()
                .expect(&err_message);
            tx.send(()).expect("Channel will be there waiting for the pool");
        });
    }

    pool.join();
}
