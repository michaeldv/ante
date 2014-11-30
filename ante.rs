// Copyright (c) 2013-2104 Michael Dvorkin
// Ante is an esoteric programming language where all you've got is a deck of cards.
//
// This is Ante implementation in Rust.

extern crate regex;
extern crate num;

use std::io::File;
use std::collections::HashMap;

use regex::Regex;
use num::bigint::BigInt;


struct Card {
    rank: u32,
    suit: u32
}

struct Ante {
    pc:     int,                    // Program counter (index within ante.code)
    line:   int,                    // Current line number.
    code:   Vec<Card>,              // Array of cards.
    vars:   HashMap<char, uint>,    // Four registers hashed by suit.
    labels: HashMap<uint, uint>,    // Labels for ante.pc to jump to.
    buffer: Vec<char>               // Buffer to collect UTF-8 character bytes.
}

impl Ante {
    fn new() -> Ante {
        let mut vars = HashMap::new();
        vars.insert('♦', 0);
        vars.insert('♥', 0);
        vars.insert('♠', 0);
        vars.insert('♣', 0);

        Ante {
            pc:     0,
            line:   0,
            code:   vec![],
            vars:   vars,
            labels: HashMap::new(),
            buffer: vec![]
        }
    }

    fn run(&mut self, filename: &str) {
        let mut file = File::open(&Path::new(filename));
        let program = file.read_to_string().unwrap();
        println!("file: {}", program);
        self.parse(program.as_slice())
    }

    // Turn source file into array of cards.
    fn parse(&mut self, program: &str) {
        // Split program blob into lines getting rid of comments and whitespaces.
        let comments = Regex::new(r"#.*$").unwrap();
        let lines: Vec<String> = program.lines().map( |line|
            comments.replace_all(line, "").as_slice().trim().to_string()
        ).collect();
    }
}


fn main() {
    println!("usage: ante filename.ante");
    Ante::new().run("hello.ante".as_slice());
}
