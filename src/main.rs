extern crate blockcounter;
extern crate num_cpus;
extern crate threadpool;

use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use std::process::Command;
use std::fs::{File, ReadDir};
use std::io::Write;

fn get_read_file(filename: &String) -> File {
    match File::open(filename) {
        Ok(file) => file,
        Err(e)   => { panic!("Error opening {}: {}", filename, e.to_string()); },
    }
}

fn get_folder(foldername: &String) -> ReadDir {
    match std::fs::read_dir(&foldername) {
        Ok(dir) => dir,
        Err(e)  => { panic!("Error opening {}: {}", foldername, e.to_string()); },
    }
}

fn get_write_file(filename: &String) -> File {
    match File::create(filename) {
        Ok(file) => file,
        Err(e)   => { panic!("Error create {}: {}", filename, e.to_string()); },
    }
}

fn check_folder(path: &String) -> Result<(), String> {
    let _file = get_folder(path);
    Ok(())
}

fn help(bin: &String) {
    println!("Usage: {} data gp [tmp]", bin);
}


fn main() {
    const GNUPLOT_SEPARATOR_NO: usize = 2;

    let args: Vec<String> = std::env::args().collect();

    for arg in &args {
        if arg == "-h" || arg == "--help" {
            help(&args[0]);
            return ();
        } else if arg == "-help" {
            println!("-help is obsolete. Please, use --help instead.");
            help(&args[0]);
            return ();
        }
    }
    
    if args.len() != 3 && args.len() != 4 {
        help(&args[0]);
        panic!("Usage: {} data gp [tmp]", args[0]);
    }

    let datafilename = &args[1];
    let gpfilename = &args[2];
    let tmpfoldername = {
        if args.len() > 3 {
            args[3].clone()
        } else {
            std::env::temp_dir()
                .into_os_string()
                .into_string()
                .unwrap()
        }
    };
    match check_folder(&tmpfoldername) {
        Err(e) => { panic!(e); },
        _      => { },
    }

    let datafile = get_read_file(datafilename);
    let blocks   = blockcounter::Blocks::new(GNUPLOT_SEPARATOR_NO, &datafile);
    let cpus_no  = num_cpus::get();

    let pool = ThreadPool::new(cpus_no);

    let (tx, _rx) = channel();
    for (index, block) in blocks.enumerate() {
        let tx = tx.clone();
        let index = index.clone();
        let gpfilename = gpfilename.clone();
        let tmpfoldername = tmpfoldername.clone();
        let block = block.clone();
        
        pool.execute(move || {
            let index_str = &index.to_string();
            let err_message = "Failed to execute GNUPlot with index {}.".to_string() + index_str;
            let tmpfilename = tmpfoldername + "/" + index_str;
            let mut tmpfile = get_write_file(&tmpfilename);
            
            match tmpfile.write_all(block.as_bytes()) {
                Err(e) => { panic!("Error writting in {}: {}", tmpfilename, e.to_string()); },
                Ok(()) => { },
            }
            match tmpfile.flush() {
                Err(e) => { panic!("Error flushing {}: {}", tmpfilename, e.to_string()); },
                Ok(()) => { },
            }
            let _status = Command::new("gnuplot")
                .args(&["-e", &format!("INDEX={}", index_str)])
                .args(&["-e", &format!("DATAFILE=\"{}\"", tmpfilename)])
                .args(&[&gpfilename])
                .status()
                .expect(&err_message);
            match std::fs::remove_file(&tmpfilename) {
                Err(e) => { panic!("Error removing {}: {}", tmpfilename, e.to_string()); },
                Ok(()) => { },
            }
            
            tx.send(()).expect("Channel will be there waiting for the pool");
        });
    }

    pool.join();
}
