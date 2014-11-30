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
    pc:     uint,                   // Program counter (index within ante.code)
    line:   uint,                   // Current line number.
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

        while self.pc < self.code.len() {
            let card = self.code[self.pc];
            self.pc += 1;
            match card.rank {
                0       => self.newline(card),
                75/*K*/ => self.jump(card),
                81/*Q*/ => continue,
                74/*J*/ => self.dump(card, true),
                10      => self.dump(card, false),
                _       => self.assign(card)
            }
        }
    }

    // Turn source file into array of cards.
    fn parse(&mut self, program: &str) {
        // Split program blob into lines getting rid of comments and whitespaces.
        let comments = Regex::new(r"#.*$").unwrap();
        let lines: Vec<String> = program.lines().map( |line|
            comments.replace_all(line, "").as_slice().trim().to_string()
        ).collect();

        //\\ DEBUG //\\
        for i in range(0, lines.len()) {
            println!("{:2}) parsing: /{}/", i, lines[i]);
        }

        // Turn source file into array of cards. Each card a struct of rank and suit.
        let card = Regex::new(r"(10|[2-9JQKA])([♦♥♠♣])").unwrap();
        for (i, line) in lines.iter().enumerate() {
            // Line number cards have zero rank.
            self.code.push(Card { rank: 0, suit: i as u32 + 1 });

            // Parse cards using regural expression. Card rank and suit characters get saved
            // as u32 runes (to cast u32 back to char use std::char::from_u32(ch).unwrap()).
            for caps in card.captures_iter(line.as_slice()) {
                let rank = caps.at(1).char_at(0);
                let suit = caps.at(2).char_at(0);
                let card = match rank {
                   '1'       => Card { rank: 10, suit: suit as u32 },
                   '2'...'9' => Card { rank: rank as u32 - 48, suit: suit as u32 },
                   _         => Card { rank: rank as u32, suit: suit as u32 }
                };
                self.code.push(card);
            }
        }

        //\\ DEBUG //\\
        for i in range(0, self.code.len()) {
            println!("{:2}) code: /{}:{}/", i, self.code[i].rank, self.code[i].suit);
        }

        // Extra pass to set up jump labels.
        let mut pc = 0u;
        while pc < self.code.len() - 1 {
            let card = self.code[pc];
            pc += 1;
            if card.rank == 81 { // 'Q'
                let mut queen = card.suit as uint;
                while pc < self.code.len() - 1 && self.code[pc].rank == 81 && self.code[pc].suit == card.suit {
                    queen += card.suit as uint;
                    pc += 1;
                }
                self.labels.insert(queen, pc);
            }
        }

        //\\ DEBUG //\\
        for (k,v) in self.labels.iter() {
            println!("label: /{} => {}/", k, v);
        }
    }

    fn newline(&self, card: Card) {
        println!("newline {}:{}", card.rank, card.suit);
    }

    fn jump(&self, card: Card) {
        println!("jump {}:{}", card.rank, card.suit);
    }

    fn assign(&self, card: Card) {
        println!("assign {}:{}", card.rank, card.suit);
    }

    fn dump(&self, card: Card, as_character: bool) {
        println!("dump {}:{} as character {}", card.rank, card.suit, as_character);
    }
}


fn main() {
    println!("usage: ante filename.ante");
    Ante::new().run("factorial.ante".as_slice());
}
