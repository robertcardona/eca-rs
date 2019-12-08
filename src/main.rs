extern crate petgraph;

use std::fmt;
use std::fmt::Write as FmtWrite;

// use std::env;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

use petgraph::unionfind::UnionFind;

// timing
use std::time::{Duration, Instant};

const RULES : [u8; 88] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 18, 19, 22, 23,
    24, 25, 26, 27, 28, 29, 30, 32, 33, 34, 35, 36, 37, 38, 40, 41, 42, 43, 44, 45, 46, 50, 51,
    54, 56, 57, 58, 60, 62, 72, 73, 74, 76, 77, 78, 90, 94, 104, 105, 106, 108, 110, 122, 126,
    128, 130, 132, 134, 136, 138, 140, 142, 146, 150, 152, 154, 156, 160, 162, 164, 168, 170,
    172, 178, 184, 200, 204, 232];

// todo : would be nice to have all the equivalence classes available too.


// poster sizes

extern crate image;

pub struct Automata {
    universe : Vec<u8>

}

pub struct ECA {
    // pub states : u8, // number of states
    // pub radius : u8, //
    // pub lattice : Vec<u32>,
    rule : u8,
    width : u32,
    height : u32,
    initial_configuration : Vec<u8>,
    current_configuration : Vec<u8>,
    universe : Vec<Vec<u8>>,
}

impl ECA {

    pub fn new(rule : u8, width : u32, height : u32) -> ECA {

        // default state
        let mut initial_configuration : Vec<u8> = vec![0; width as usize];
        initial_configuration[(width / 2) as usize] = 1;

        return ECA {
            rule : rule,
            width : width,
            height : height,
            initial_configuration : initial_configuration.clone(),
            current_configuration : initial_configuration,
            universe : Vec::new(),
        };
    }

    pub fn generate(&mut self, height : u32) {
        // println!{"{}", self};
        self.universe.push(self.current_configuration.clone());

        for i in 0..(height - 1) {
            // println!("{}", i);
            self.current_configuration = self.get_next_configuration();
            self.universe.push(self.current_configuration.clone());
            // println!{"{}", self};
        }
    }

    pub fn save_to_csv(&self) -> Result<(), Box<dyn std::error::Error>>{
        // This assumes you've run generate.

        // let mut csv_filename = String::new();
        // writeln!(&mut csv_filename, "./rule{}|size{}|seed{}.csv", self.rule, self.width, 1)?;

        let csv_filename = format!("./rule{}length{}.csv", self.rule, self.width);

        let mut universe_as_string : String = String::new();

        for configuration in &self.universe {
            for (index, cell) in configuration.iter().enumerate() {
                if *cell == 1 {
                    universe_as_string.push('1');
                    // println!("cell : {}", 1);
                } else {
                    universe_as_string.push('0');
                    // println!("cell : {}", 0);
                }

                if index == (self.width - 1) as usize {
                    universe_as_string.push('\n');
                } else {
                    universe_as_string.push(',');
                }
            }
        }

        // println!("{}", universe_as_string);
        std::fs::write(csv_filename, universe_as_string).unwrap();

        return Ok(());
    }

    pub fn reset(&mut self) {
        self.universe =  Vec::new();
        self.current_configuration = self.initial_configuration.clone();
    }

    fn get_current_configuration(&self) -> String {
        let mut configuration = String::new();

        for cell in &self.current_configuration {
            configuration.push_str(&cell.to_string());
        }

        return configuration;
    }

    pub fn get_flattened_universe(&self) -> Vec<u8> {
        let mut universe : Vec<u8> = Vec::new();

        for row in &self.universe {
            for cell in row {
                if *cell == 0 {
                    universe.push(0xff);
                    universe.push(0xff);
                    universe.push(0xff);
                    universe.push(0xff);
                } else {
                    universe.push(0);
                    universe.push(0);
                    universe.push(0);
                    universe.push(0xff);
                }
            }
        }

        return universe;
    }

    pub fn generate_connected_components(&self) {

    }

    fn rule_lookup(&self, a : u8, b : u8, c : u8) -> u8 {
        let mask_bit : u8 = a << 2 | b << 1 | c;
        // println!{"a {}, b {}, c {}, bit_mask : {}", a, b, c, mask_bit};
        return (self.rule & (1 << mask_bit)) >> mask_bit;
    }

    fn get_next_configuration(&self) -> Vec<u8> {
        let mut next_configuration : Vec<u8> = vec![0; self.width as usize];

        let boundary_is_periodic : bool = false;

        for (index, state) in self.current_configuration.iter().enumerate() {
            let next_state : u8;

            if index == 0 {
                // println!("index = 0");
                next_state = self.rule_lookup(
                    if boundary_is_periodic
                    {
                        self.current_configuration[(self.width - 1) as usize]
                    } else {
                        0
                    },
                    self.current_configuration[0],
                    self.current_configuration[1]
                );
            } else if index == (self.width - 1) as usize {
                // println!("index = width - 1");
                next_state = self.rule_lookup(
                    self.current_configuration[(self.width - 2) as usize],
                    self.current_configuration[(self.width - 1) as usize],
                    if boundary_is_periodic {
                        self.current_configuration[0]
                    } else {
                        0
                    }
                )
            } else {
                next_state = self.rule_lookup(
                    self.current_configuration[index - 1],
                    self.current_configuration[index],
                    self.current_configuration[index + 1]
                );
            }
            next_configuration[index] = next_state;

            // println!("{}, {}", index, state);
        }

        return next_configuration;
    }


}

impl fmt::Display for ECA {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        return write!(f, "Rule : {} | current_configuration : {} ", self.rule, self.get_current_configuration());
    }
}

fn generate_rule(rule : u8, width : u32, height : u32, generate_csv : bool, generate_images : bool, resize : bool, ccl : bool) {
    // println!("{}", rule);

    // let mut automata = ECA::new(rule as u8, width);
    // automata.generate(height);

    // let mut filename = String::new();
    // filename.push_str("./rule");
    // filename.push_str(&rule.to_string());
    // filename.push_str("length");
    // filename.push_str(&width.to_string());
    // filenlf.currename.push_str(".png");

    let filename = format!("./rule{}length{}.png", &rule.to_string(), &width.to_string());

    let mut automata = ECA::new(rule as u8, width, height);
    automata.generate(height);

    if generate_images {
        let buffer = automata.get_flattened_universe();
        image::save_buffer(&filename, buffer.as_slice(), width, height, image::RGBA(8)).unwrap();

        if resize {
            // resize
            let mut im = image::open(&filename).unwrap();
            // let mut fout = File::open(filename);
            let fout = &mut File::create(&Path::new(&filename)).unwrap();
            // Write the contents of this image to the Writer in PNG format.
            // im.thumbnail(800, 600).write_to(fout, image::PNG).unwrap();
            im.resize(1063, 1375, image::FilterType::Nearest).write_to(fout, image::PNG).unwrap();
        }
    }

    if generate_csv {
        automata.save_to_csv().unwrap();
    }

}

fn generate_all_rules(width : u32, height : u32, generate_csv : bool, generate_images : bool, resize : bool, ccl : bool) {
    for rule in RULES.iter() {
        generate_rule(*rule, width, height, generate_csv, generate_images, resize, ccl);
    }

}

fn main() {
   let start = Instant::now();

    // generate_all_rules();
    let width : u32 = 3600;
    let height : u32 = 5400;

    let generate_csv : bool = false;
    let generate_images : bool = true;
    let resize : bool = false;
    let connected_component_labelling : bool = true;

    generate_all_rules(width, height, generate_csv, generate_images, resize, connected_component_labelling);

    // generate_rule(129, width, height, generate_csv, generate_images, resize, connected_component_labelling);

    // let mut automata = ECA::new(86 as u8, width as u32);
    // automata.generate(width as u32);
    //
    // let mut buffer = automata.get_flattened_universe();
    // println!("buffer : {}", buffer.len());
    //
    // image::save_buffer("./image.png", buffer.as_slice(), width, width, image::RGBA(8)).unwrap();

    // test union-find

    let n = 8;
    let mut u = UnionFind::<u32>::new((width * height) as usize);

    for i in 0..n {
        println!("{}", u.find(i as u32));
        // assert_eq!(u.find(i), i);
        // assert_eq!(u.find_mut(i), i);
        // assert!(!u.union(i, i));
    }

    u.union(0, 1);
    println!("{}", u.find(0));
    println!("{}", u.find(1));

    assert_eq!(u.find(0), u.find(1));
    u.union(1, 3);
    u.union(1, 4);
    u.union(4, 7);
    assert_eq!(u.find(0), u.find(3));
    assert_eq!(u.find(1), u.find(3));
    assert!(u.find(0) != u.find(2));
    assert_eq!(u.find(7), u.find(0));
    u.union(5, 6);
    assert_eq!(u.find(6), u.find(5));
    assert!(u.find(6) != u.find(7));

    println!("runtime : {} ns", start.elapsed().as_nanos());
}
