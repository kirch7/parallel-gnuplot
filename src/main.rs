#[macro_use] extern crate clap;
extern crate blockcounter;
extern crate num_cpus;
extern crate threadpool;

#[cfg(not(windows))] extern crate isatty;

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

fn are_all_none(v: &Vec<Option<String>>) -> bool {
    let mut some_some = false;
    for elem in v {
        if elem.is_some() {
            some_some = true;
            break;
        }
    }
    !some_some
}

fn run<S>(iters: &mut [blockcounter::Blocks<S>], gpfilename: &Option<String>, tmpfoldername: &String, jobs: usize, do_delete: bool)
    where std::io::BufReader<S> : std::io::BufRead {

    use threadpool::ThreadPool;
    use std::sync::mpsc::channel;
    use std::process::Command;
    use std::io::Write;

    let iters_no = iters.len();

    let mut count = 0usize;
    
    let mut do_delete = do_delete;

    let pool = ThreadPool::new(jobs);
    let (tx, _rx) = channel();
    loop {
        let mut strings: Vec<Option<String>> = Vec::new();
        for iters_index in 0..iters_no {
            strings.push(iters[iters_index].nth(0));
        }
        let strings = strings;
        if are_all_none(&strings) {
            break;
        }
        
        let tx = tx.clone();
        let index = count.clone();
        count += 1;
        let gpfilename = gpfilename.clone();
        let tmpfoldername = tmpfoldername.clone();
        
        pool.execute(move || {
            let err_message = "Failed to execute GNUPlot with index {}.".to_string() + &index.to_string();
            let mut tmpfilename_vec = Vec::new();
            for iters_index in 0..iters_no {
                let tmpfoldername = tmpfoldername.clone();;
                let tmpfilename = tmpfoldername + &format!("/tmp{}_index{}", iters_index, index);
                tmpfilename_vec.push(tmpfilename.clone());
                let mut tmpfile = get_write_file(&tmpfilename);
                let block = strings[iters_index].clone();
                if block.is_none() { continue; }
                match tmpfile.write_all(block.unwrap().as_bytes()) {
                    Err(e) => { panic!("Error writting in {}: {}", tmpfilename, e.to_string()); },
                    Ok(()) => { },
                }
                match tmpfile.flush() {
                    Err(e) => { panic!("Error flushing {}: {}", tmpfilename, e.to_string()); },
                    Ok(()) => { },
                }
                
            }

            if let Some(gpfilename) = gpfilename {
                let args = {
                    let mut vec = Vec::new();
                    for (index, tmpfilename) in tmpfilename_vec.iter().enumerate() {
                        vec.push("-e".to_string());
                        let arg = format!("DATAFILE{}=\"{}\"", index, tmpfilename);
                        vec.push(arg.clone());
                    }
                    vec
                };
                let _status = Command::new("gnuplot")
                    .args(&["-e", &format!("INDEX={}", index)])
                    .args(args.as_slice())
                    .args(&[&gpfilename])
                    .status()
                    .expect(&err_message);
            } else {
                do_delete = false;
            }
            if do_delete {
                for tmpfilename in &tmpfilename_vec {
                    match std::fs::remove_file(&tmpfilename) {
                        Err(e) => { panic!("Error removing {}: {}", tmpfilename, e.to_string()); },
                        Ok(()) => { },
                    }
                }
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

fn get_datafilename_vec<'a, 'b>(args: &'b clap::ArgMatches<'a>, is_a_tty: bool) -> Option<Vec<&'b str>> {
    let datafilename_vec: Option<Vec<&'b str>> = match is_a_tty {
        true  => {
            let ds: Option<clap::Values<'b>> = args
                .values_of("DATAS");
            if ds.is_some() {
                let ds: Vec<&'b str> = ds
                    .unwrap()
                    .collect();
                if ds.len() == 0 {
                    None
                } else {
                    Some(ds)
                }
            } else {
                None
            }
        },
        false => None,
    };

    if is_a_tty && datafilename_vec.is_none() {
        panic!("You must specify at least one data file.");
    }
    if !is_a_tty && datafilename_vec.is_some() {
        panic!("You should not specify any data file.");
    }
    
    datafilename_vec
}

fn main() {
    const GNUPLOT_SEPARATOR_NO: usize = 2;

    let base_yaml     = load_yaml!("en_base.yml");

    let is_a_tty = stdin_isatty();
    let args_matches = clap::App::from_yaml(base_yaml);
    let args_matches = args_matches
        .version(crate_version!())
        .author(crate_authors!())
        .get_matches();

    let know = args_matches
        .is_present("KNOW");

    let datafilename_vec = get_datafilename_vec(&args_matches, is_a_tty);
    
    let gpfilename = match args_matches.is_present("GNUPLOTSCRIPT") {
        true  => {
            let gp = args_matches
                .value_of("GNUPLOTSCRIPT")
                .unwrap()
                .to_string();
            let _ = is_gp_script_ok(&gp);
            Some(gp)
        },
        false => {
            if !know {
                eprintln!("You skipped GNUplot script name. I will continue after a while.");
                std::thread::sleep(std::time::Duration::from_secs(7));
            } else {
                eprintln!("You skipped GNUplot script name.");
            }
            None
        },
    };

    let do_delete = !args_matches
        .is_present("KEEPDATA");

    println!("{:?}", args_matches.value_of("TMPFOLDER"));
    let tmpfoldername = match args_matches
        .value_of("TMPFOLDER") {
            Some(s) => s.to_string(),
            None    => {
                if !do_delete || gpfilename.is_none() {
                    if !know {
                        eprintln!("You are using default temporary folder. I will continue after a while.");
                        std::thread::sleep(std::time::Duration::from_secs(7));
                    } else {
                        eprintln!("You are using default temporary folder.");
                    }
                }
                std::env::temp_dir()
                    .into_os_string()
                    .into_string()
                    .unwrap()
            },
        };

    match check_folder(&tmpfoldername) {
        Err(e) => { panic!(e); },
        _      => { },
    }

    let jobs_no = match args_matches.value_of("JOBS") {
        Some(n) => {
            let jobs_no = n.parse::<usize>().unwrap();
            if jobs_no > num_cpus::get() {
                if !know {
                    eprintln!("You are using too many threads. I will continue after a while.");
                    std::thread::sleep(std::time::Duration::from_secs(7));
                } else {
                    eprintln!("You are using too many threads.");
                }
            }
            jobs_no
        },
        None    => num_cpus::get(),
    };

    match is_a_tty {
        true  => {
            let mut iters: Vec<blockcounter::Blocks<File>> = datafilename_vec
                .unwrap()
                .iter()
                .map(|datafilename| get_read_file(&datafilename.to_string()))
                .map(|datafile| blockcounter::Blocks::new(GNUPLOT_SEPARATOR_NO, datafile))
                .collect();
            run(&mut iters, &gpfilename, &tmpfoldername, jobs_no, do_delete);
        },
        false => {
            use std::io::Read;
            let stdin_ = std::io::stdin();
            let mut s = String::new();
            let _ = stdin_
                .lock()
                .read_to_string(&mut s)
                .unwrap();
            let mut v = vec![blockcounter::Blocks::new(GNUPLOT_SEPARATOR_NO, s.as_bytes())];
            run(&mut v, &gpfilename, &tmpfoldername, jobs_no, do_delete);
        },
    };
}
