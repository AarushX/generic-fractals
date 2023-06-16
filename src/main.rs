use basemath::*;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::thread;
use serde_derive::Deserialize;
use std::process::exit;
use toml;

#[derive(Deserialize)]
struct Data {
    config: Config,
    advanced: Advanced,
}
#[derive(Deserialize)]
struct Config {
    width: u32,
    height: u32,
}

#[derive(Deserialize)]
struct Advanced {
    seed: CompStruct,
    topleft: CompStruct,
    bottomright: CompStruct,
    bound: f64,
    iterate: u32,
    threads: u32,
}

#[derive(Deserialize)]
struct CompStruct {
    r: f64,
    i: f64,
}


static HEX_GS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

// Change me to override thread settings
static ADVANCED_MODE: bool = false;
static CONFIG_FILE: &str = "./config.toml";

fn main() {


    let contents = match read_to_string(CONFIG_FILE) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Could not read file `{}`", CONFIG_FILE);
            exit(1);
        }
    };

    let data: Data = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Unable to load data from `{}`", CONFIG_FILE);
            exit(1);
        }
    };

    let width: u32 = data.config.width;
    let height: u32 = data.config.height;

    let advanced_data = data.advanced;

    let _ = create_dir_all("./plots/build");
    let seed: Comp = Comp { r: advanced_data.seed.r, i: advanced_data.seed.i };
    let topleft: Comp = Comp { r: -2.0, i: 2.0 };
    let bottomright: Comp = Comp { r: 2.0, i: -2.0 };
    let bound: f64 = 2.0;
    
    let iterate: usize = 128;
    let mut threads: u32 = 2;
    
    
    if !ADVANCED_MODE {
        threads = match thread::available_parallelism() {
            Ok(res) => res.get().try_into().unwrap(),
            Err(error) => {
                println!(
                    "Thread calculation failed. Using {} threads. {}",
                    threads, error
                );
                2
            }
        };
    }

    multi_p(
        seed,
        topleft,
        bottomright,
        bound,
        width,
        height,
        iterate,
        threads,
    );
}

#[allow(dead_code)]
fn multi_i(
    par: Comp,
    topleft: Comp,
    bottomright: Comp,
    bound: f64,
    width: u32,
    height: u32,
    iterate: usize,
    threads: u32,
) {
    let mut header = File::create("./plots/build/0.npxl").expect("cannot create cur file");
    let first = format!("{} {}\n", width, height) + "16 1\n";
    header
        .write_all(first.as_bytes())
        .expect("cannot write header");

    let separation: f64 = (topleft.i - bottomright.i) * 0.125;
    let mut loc_tl: Comp = topleft;
    let mut loc_br: Comp = Comp {
        r: bottomright.r,
        i: topleft.i - separation,
    };
    let mut allthr: Vec<_> = Vec::new();
    for parallel in 1..threads + 1 {
        allthr.push(thread::spawn(move || {
            ispace(
                par,
                loc_tl,
                loc_br,
                bound,
                width,
                height / threads,
                iterate,
                parallel as usize,
            );
        }));
        loc_tl.i -= separation;
        loc_br.i -= separation;
    }
    for each in allthr {
        each.join().unwrap();
    }
}
#[allow(dead_code)]
fn multi_p(
    seed: Comp,
    topleft: Comp,
    bottomright: Comp,
    bound: f64,
    width: u32,
    height: u32,
    iterate: usize,
    threads: u32,
) {
    let mut header = File::create("./plots/build/0.npxl").expect("cannot create cur file");
    let first = format!("{} {}\n", width, height) + "16 1\n";
    header
        .write_all(first.as_bytes())
        .expect("cannot write header");

    let separation: f64 = (topleft.i - bottomright.i) * 0.125;
    let mut loc_tl: Comp = topleft;
    let mut loc_br: Comp = Comp {
        r: bottomright.r,
        i: topleft.i - separation,
    };
    let mut allthr: Vec<_> = Vec::new();
    for parallel in 1..threads + 1 {
        allthr.push(thread::spawn(move || {
            pspace(
                seed,
                loc_tl,
                loc_br,
                bound,
                width,
                height / threads,
                iterate,
                parallel as usize,
            );
        }));
        loc_tl.i -= separation;
        loc_br.i -= separation;
    }
    for each in allthr {
        each.join().unwrap();
    }
}

#[allow(dead_code)]
fn ispace(
    par: Comp,
    topleft: Comp,
    bottomright: Comp,
    bound: f64,
    width: u32,
    height: u32,
    iterate: usize,
    num: usize,
) {
    let name: String = format!("./plots/build/{num}.npxl");
    let path = Path::new(name.as_str());
    let mut file = File::create(&path).unwrap();

    let formula = |z: Comp, c: Comp| z * z + c;

    let rstep: f64 = (bottomright.r - topleft.r) / width as f64;
    let istep: f64 = (topleft.i - bottomright.i) / height as f64;
    let mut crnt: Comp = topleft + Comp::new(0.5 * rstep, -0.5 * istep);
    let mut counter: usize;
    let mut valstring: String;
    let mut z: Comp;

    for _ in 0..height {
        valstring = String::new();
        crnt.r = topleft.r + 0.5 * rstep;
        for _ in 0..width {
            counter = 0;
            z = crnt;
            loop {
                if counter == iterate {
                    counter = 0;
                    break;
                }
                if z.r * z.r + z.i * z.i > bound * bound {
                    break;
                }
                z = formula(z, par);
                counter += 1;
            }
            valstring = format!("{valstring}{}", HEX_GS[counter * 16 / iterate]);
            crnt.r += rstep;
        }
        valstring += "\n";
        file.write_all(valstring.as_bytes())
            .expect("cannot write line");
        crnt.i -= istep;
    }
}
#[allow(dead_code)]
fn pspace(
    seed: Comp,
    topleft: Comp,
    bottomright: Comp,
    bound: f64,
    width: u32,
    height: u32,
    iterate: usize,
    num: usize,
) {
    let name: String = format!("./plots/build/{num}.npxl");
    let path = Path::new(name.as_str());
    let mut file = File::create(&path).unwrap();

    let formula = |z: Comp, c: Comp| z * z + c;

    let rstep: f64 = (bottomright.r - topleft.r) / width as f64;
    let istep: f64 = (topleft.i - bottomright.i) / height as f64;
    let mut crnt: Comp = topleft + Comp::new(0.5 * rstep, -0.5 * istep);
    let mut counter: usize;
    let mut valstring: String;
    let mut z: Comp;
    let mut c: Comp;

    for _ in 0..height {
        valstring = String::new();
        crnt.r = topleft.r + 0.5 * rstep;
        for _ in 0..width {
            counter = 0;
            z = seed;
            c = crnt;
            loop {
                if counter == iterate {
                    counter = 0;
                    break;
                }
                if z.r * z.r + z.i * z.i > bound * bound {
                    break;
                }
                z = formula(z, c);
                counter += 1;
            }
            valstring = format!("{valstring}{}", HEX_GS[counter * 16 / iterate]);
            crnt.r += rstep;
        }
        valstring += "\n";
        file.write_all(valstring.as_bytes())
            .expect("cannot write line");
        crnt.i -= istep;
    }
}
