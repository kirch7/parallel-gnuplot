#[macro_use] extern crate clap;
extern crate blockcounter;
extern crate num_cpus;
extern crate threadpool;
extern crate isatty;

use std::fs::{File, ReadDir};

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

fn is_gp_script_ok(gpfilename: &String) -> bool {
    use std::io::{BufRead, BufReader};
    use std::collections::HashMap;

    const GP_VARS: &[&str] = &["INDEX", "DATAFILE"];
    let mut hash = HashMap::new();
    
    let file = get_read_file(gpfilename);
    let file = BufReader::new(file);
    for line in file.lines() {
        let line = line.unwrap();
        for gp_var in GP_VARS {
            if line.contains(gp_var) {
                let _ = hash
                    .entry(gp_var)
                    .or_insert(true);
            }
        }
    }

    for gp_var in GP_VARS {
        let _ = hash
            .entry(gp_var)
            .or_insert(false);
    }

    
    let mut ok = true;
    for (gp_var, has) in hash {
        if !has {
            eprintln!("Maybe you want to use {} in {}.", gp_var, gpfilename);
            ok = false
        }
    }

    ok
}

fn run<S>(iter: blockcounter::Blocks<S>, gpfilename: &String, tmpfoldername: &String, jobs: usize)
    where std::io::BufReader<S> : std::io::BufRead {

    use threadpool::ThreadPool;
    use std::sync::mpsc::channel;
    use std::process::Command;
    use std::io::Write;
    
    let pool = ThreadPool::new(jobs);

    let (tx, _rx) = channel();
    for (index, block) in iter.enumerate() {
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

#[cfg(not(windows))]
#[inline(always)]
fn stdin_isatty() -> bool {
    isatty::stdin_isatty()
}

#[cfg(windows)]
#[inline(always)]
fn stdin_isatty() -> bool {
    true
}

fn main() {
    let filtered_yaml = load_yaml!("en.yml");
    let base_yaml     = load_yaml!("en_base.yml");

    let is_a_tty = stdin_isatty();
    let args_matches = match is_a_tty {
        true  => clap::App::from_yaml(base_yaml),
        false => clap::App::from_yaml(filtered_yaml),
    };
    let args_matches = args_matches
        .version(crate_version!())
        .author(crate_authors!())
        .get_matches();
    let datafilename = match is_a_tty {
        true  => Some(args_matches
                      .value_of("DATA")
                      .unwrap()),
        false => None,
    };
    
    const GNUPLOT_SEPARATOR_NO: usize = 2;

    let gpfilename = args_matches
        .value_of("GNUPLOTSCRIPT")
        .unwrap()
        .to_string();
    let _ = is_gp_script_ok(&gpfilename);
    let tmpfoldername = args_matches
        .value_of("TMPFOLDER")
        .unwrap_or(&std::env::temp_dir()
                   .into_os_string()
                   .into_string()
                   .unwrap())
        .to_string();
    match check_folder(&tmpfoldername) {
        Err(e) => { panic!(e); },
        _      => { },
    }

    let jobs_no = match args_matches.value_of("JOBS") {
        Some(n) => {
            let jobs_no = n.parse::<usize>().unwrap();
            if jobs_no > num_cpus::get() {
                eprintln!("You are using too many threads. I will continue after a while.");
                std::thread::sleep(std::time::Duration::from_secs(7));
            }
            jobs_no
        },
        None    => num_cpus::get(),
    };
    
    match is_a_tty {
        true  => {
            let datafile = get_read_file(&datafilename.unwrap().to_string());
            run(blockcounter::Blocks::new(GNUPLOT_SEPARATOR_NO, &datafile), &gpfilename, &tmpfoldername, jobs_no);
        },
        false => {
            use std::io::Read;
            let stdin_ = std::io::stdin();
            let mut s = String::new();
            let _ = stdin_
                .lock()
                .read_to_string(&mut s)
                .unwrap();
            run(blockcounter::Blocks::new(GNUPLOT_SEPARATOR_NO, s.as_bytes()), &gpfilename, &tmpfoldername, jobs_no);
        },
    };
}
