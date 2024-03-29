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

enum PixelType {
    Background,
    Foreground
}


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
    initial_configuration : Vec<u32>,
    universe : Vec<u32>
}

impl ECA {

    pub fn new(rule : u8, width : u32, height : u32) -> ECA {

        // default state
        let mut initial_configuration : Vec<u32> = vec![0; width as usize];

        // todo, rewrite new to pass in the initial config
        initial_configuration[(width / 2) as usize] = 1;

        // initialize universe with initial configuration
        let mut universe : Vec<u32> = vec![0; (width * height) as usize];

        for (index, cell) in initial_configuration.iter().enumerate() {
            universe[index] = *cell;
        }

        return ECA {
            rule : rule,
            width : width,
            height : height,
            initial_configuration : initial_configuration.clone(),
            universe : universe
        };
    }

    pub fn get_value(&self, row_index : u32, column_index : u32) -> u32{
        return self.universe[(row_index * self.width + column_index) as usize];
    }

    pub fn set_value(&mut self, row_index : u32, column_index : u32, value : u32) {
        self.universe[(row_index * self.width + column_index) as usize] = value;
    }

    fn increase_generation(&mut self, row_index : u32) {
        for cell_index in 0..(self.width) {
            // we look behind to the previous row to generate this row
            let radius : u8 = self.get_radius(row_index - 1, cell_index);
            let new_bit : u8 = self.rule_lookup(radius);
            self.set_value(row_index, cell_index, new_bit as u32);
        }
    }

    pub fn generate(&mut self) {
        for row_index in 1..self.height {
            self.increase_generation(row_index);
        }
    }

    fn get_radius(&self, row_index : u32, column_index : u32) -> u8 {

         let is_periodic : bool = false; // todo : eventually pass through

         let mut a : u32 = self.get_value(
             row_index,
             (column_index +
                 if column_index == 0 {self.width} else {0}) - 1
                 % self.width);
         let b : u32 = self.get_value(row_index, column_index);
         let mut c : u32 = self.get_value(row_index, (column_index + 1) % self.width);

         if is_periodic == false {
             if (column_index +
                 if column_index == 0 {self.width} else {0} - 1) % self.width == 0 {
                 a = (a ^ a) & 1;
             }
             if (column_index + 1) % self.width == self.width - 1 {
                 c = (c ^ c) & 1;
             }
         }

         return (a << 2 | b << 1 | c) as u8; // the new rule, as a number
    }

    fn rule_lookup(&self, rule : u8) -> u8{
        return (self.rule & (1 << rule)) >> rule; // todo : maybe rename rule to shift
    }

    fn get_universe_str(&self) -> String {
        let mut universe = String::new();

        for (index, cell) in self.universe.iter().enumerate() {
            universe.push_str(&cell.to_string());
            if index % self.width as usize == 0 {
                universe.push('\n');
            }
        }

        return universe;
    }

    pub fn get_flattened_universe(&self) -> Vec<u8> {
        let mut universe : Vec<u8> = Vec::new();


        for cell in &self.universe {

            let a : u8 = (*cell & 255) as u8;
            let b : u8 = (*cell & (255 << 8)) as u8;
            let c : u8 = (*cell & (255 << 16)) as u8;
            let d : u8 = (*cell & (255 << 24)) as u8;

            if *cell == 0 {
                universe.push(0xff);
                universe.push(0xff);
                universe.push(0xff);
                universe.push(0xff);
            } else {
                universe.push(a);
                universe.push(b);
                universe.push(c);
                universe.push(0xff);
            }
        }
        return universe;
    }

    pub fn generate_connected_components(&mut self) {
        self.universe = get_connected_components(self.width, self.height, &self.universe);
    }

    pub fn save_to_csv(&self) -> Result<(), Box<dyn std::error::Error>>{
        // This assumes you've run generate.

        // let mut csv_filename = String::new();
        // writeln!(&mut csv_filename, "./rule{}|size{}|seed{}.csv", self.rule, self.width, 1)?;

        let csv_filename = format!("./rule{}length{}.csv", self.rule, self.width);

        let mut universe_as_string : String = String::new();

        for (index, cell) in self.universe.iter().enumerate() {
            if *cell != 0 {
                universe_as_string.push_str(&cell.to_string());
                // println!("cell : {}", 1);
            } else {
                universe_as_string.push('0');
                // println!("cell : {}", 0);
            }

            if index % self.width as usize == (self.width - 1) as usize {
                universe_as_string.push('\n');
            } else {
                universe_as_string.push(',');
            }
        }


        // println!("{}", universe_as_string);
        std::fs::write(csv_filename, universe_as_string).unwrap();

        return Ok(());
    }

    pub fn reset(&mut self) {
        // initialize universe with initial configuration
        let mut universe : Vec<u32> = vec![0; (self.width * self.height) as usize];

        for (index, cell) in self.initial_configuration.iter().enumerate() {
            universe[index] = *cell;
        }

        self.universe = universe;
    }

}

impl fmt::Display for ECA {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        return write!(f, "Rule : {} | universe : \n{} ", self.rule, self.get_universe_str());
    }
}

fn generate_rule(rule : u8, width : u32, height : u32, generate_csv : bool, generate_images : bool, resize : bool, ccl : bool) {

    let filename = format!("./rule{}length{}.png", &rule.to_string(), &width.to_string());

    let mut automata = ECA::new(rule as u8, width, height);
    automata.generate();

    if ccl {
        automata.generate_connected_components();
        //get_connected_components(width, height, &automata.universe);
    }

    if generate_images {
        let buffer = automata.get_flattened_universe();
        image::save_buffer(&filename, buffer.as_slice(), width, height, image::RGBA(8)).unwrap();

        if resize { // extremely slow
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

    // inequivalent rules
    // for rule in RULES.iter() {
    //     generate_rule(*rule, width, height, generate_csv, generate_images, resize, ccl);
    // }

    for rule in 0..256 {
        generate_rule(rule as u8, width, height, generate_csv, generate_images, resize, ccl);
    }
}

fn get_connected_components(width : u32, height : u32, universe : &Vec<u32>) -> Vec<u32> {

    // assumes universe is flattened based on width, height

    let mut uf = UnionFind::<u32>::new((width * height) as usize);

    for row_index in 0..height {
        for cell_index in 0..width {
            // println!("row_index:{}|cell_index:{}|total_index:{}", row_index, cell_index, (row_index * width + cell_index));
            let cell : u32 = universe[(row_index * width + cell_index) as usize];
            if cell == 1 {
                // foreground
                // println!{"foreground"};

                let mut labels : Vec<u32> = Vec::new();

                // get neighboring labels
                // todo : should be able to rewrite all of this to reuse offset = (x, y) code

                if row_index != 0 { // row above
                    // north
                    let north_cell : u32 = universe[((row_index - 1) * width + cell_index) as usize];
                    // println!("north_cell:{}", north_cell);
                    if north_cell == 1 { // foreground
                        let label : u32 = uf.find((row_index - 1) * width + cell_index);
                        // println!{"\tlabel:{}", label};
                        labels.push(label);
                    }
                }

                if cell_index != 0 { // left column
                    // west
                    let west_cell : u32 = universe[(row_index * width + (cell_index - 1)) as usize];

                    if west_cell == 1 { // foreground
                        let label : u32 = uf.find(row_index * width + (cell_index - 1));

                        labels.push(label);
                    }
                }

                if cell_index != width - 1 { // right column
                    let north_cell : u32 = universe[(row_index * width + (cell_index + 1)) as usize];

                    if north_cell == 1 { // foreground
                        let label : u32 = uf.find(row_index * width + (cell_index + 1));

                        labels.push(label);
                    }
                }

                if row_index != height - 1 { // row below
                    // south
                    let south_cell : u32 = universe[((row_index + 1) * width + cell_index) as usize];

                    if south_cell == 1 { // foreground
                        let label : u32 = uf.find((row_index + 1) * width + cell_index);

                        labels.push(label);
                    }
                }

                for label in &labels {
                    uf.union(row_index * width + cell_index, *label);
                    // println!{"\tlabel:{}", label};
                }

            } else {
                // background
                // println!{"background"};
                uf.union(0, row_index * width + cell_index);
            }
        }
    }
    // return Vec::new();

    // let mut universe_str = String::new();
    let universe_labelled = uf.into_labeling();
    //
    // for (index, cell) in universe_labelled.iter().enumerate() {
    //     universe_str.push_str(&cell.to_string());
    //
    //     if index % width as usize == (width - 1) as usize {
    //         universe_str.push('\n');
    //     } else {
    //         universe_str.push(',');
    //     }
    // }
    // println!("{}", universe_str);

    return universe_labelled;
}

fn main() {
   let start = Instant::now();

    // generate_all_rules();
    let width : u32 = 3600;
    let height : u32 = 5400;

    let generate_csv : bool = true;
    let generate_images : bool = true;
    let resize : bool = false;
    let connected_component_labelling : bool = true;

    // generate_all_rules(width, height, generate_csv, generate_images, resize, connected_component_labelling);

    generate_rule(129, width, height, generate_csv, generate_images, resize, connected_component_labelling);

    generate_rule(169, width, height, generate_csv, generate_images, resize, connected_component_labelling);

    // let mut automata = ECA::new(86 as u8, width as u32);
    // automata.generate(width as u32);
    //
    // let mut buffer = automata.get_flattened_universe();
    // println!("buffer : {}", buffer.len());
    //
    // image::save_buffer("./image.png", buffer.as_slice(), width, width, image::RGBA(8)).unwrap();

    // test union-find
    //
    // let n = 8;
    // let mut u = UnionFind::<u32>::new((width * height) as usize);
    //
    // for i in 0..n {
    //     println!("{}", u.find(i as u32));
    //     // assert_eq!(u.find(i), i);
    //     // assert_eq!(u.find_mut(i), i);
    //     // assert!(!u.union(i, i));
    // }
    //
    // u.union(0, 1);
    // println!("{}", u.find(0));
    // println!("{}", u.find(1));
    //
    // assert_eq!(u.find(0), u.find(1));
    // u.union(1, 3);
    // u.union(1, 4);
    // u.union(4, 7);
    // assert_eq!(u.find(0), u.find(3));
    // assert_eq!(u.find(1), u.find(3));
    // assert!(u.find(0) != u.find(2));
    // assert_eq!(u.find(7), u.find(0));
    // u.union(5, 6);
    // assert_eq!(u.find(6), u.find(5));
    // assert!(u.find(6) != u.find(7));

    println!("runtime : {} ns", start.elapsed().as_nanos());
}
