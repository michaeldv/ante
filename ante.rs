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

static A: u32 = 65;
static J: u32 = 74;
static Q: u32 = 81;
static K: u32 = 75;

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
}

impl Ante {
    fn new(filename: &str) -> Ante {
        let mut vars = HashMap::new();
        vars.insert('♦', 0);
        vars.insert('♥', 0);
        vars.insert('♠', 0);
        vars.insert('♣', 0);

        let code = Ante::parse(filename);
        let labels = Ante::resolve(&code);

        Ante {
            pc:     0,
            line:   0,
            code:   code,
            vars:   vars,
            labels: labels
        }
    }

    fn run(&mut self) -> &Ante {
        while self.pc < self.code.len() {
            let card = self.code[self.pc];
            self.pc += 1;
            match card.rank {
                0  => self.newline(card),
                K  => self.jump(card),
                Q  => continue,
                J  => self.dump(card, true),
                10 => self.dump(card, false),
                _  => self.assign(card)
            };
        }
        self
    }

    // Turn source file into array of cards.
    fn parse(filename: &str) -> Vec<Card> {
        let mut file = File::open(&Path::new(filename));
        let program = file.read_to_string().unwrap();
        println!("file: {}", program);

        // Split program blob into lines getting rid of comments and whitespaces.
        let comments = Regex::new(r"#.*$").unwrap();
        let lines: Vec<String> = program.as_slice().lines().map( |line|
            comments.replace_all(line, "").as_slice().trim().to_string()
        ).collect();

        //\\ DEBUG //\\
        for i in range(0, lines.len()) {
            println!("{:2}) parsing: /{}/", i, lines[i]);
        }

        // Turn source file into array of cards. Each card a struct of rank and suit.
        let mut code: Vec<Card> = vec![];
        let card = Regex::new(r"(10|[2-9JQKA])([♦♥♠♣])").unwrap();
        for (i, line) in lines.iter().enumerate() {
            // Line number cards have zero rank.
            code.push(Card { rank: 0, suit: i as u32 + 1 });

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
                code.push(card);
            }
        }

        //\\ DEBUG //\\
        for i in range(0, code.len()) {
            println!("{:2}) code: /{}:{}/", i, code[i].rank, code[i].suit);
        }
        code
    }

    // Extra pass to set up jump labels.
    fn resolve(code: &Vec<Card>) -> HashMap<uint, uint> {
        let mut pc = 0u;
        let mut labels: HashMap<uint, uint> = HashMap::new();

        while pc < code.len() - 1 {
            let card = code[pc];
            pc += 1;
            if card.rank == Q {
                let mut queen = card.suit as uint;
                while pc < code.len() && code[pc].rank == Q && code[pc].suit == card.suit {
                    queen += card.suit as uint;
                    pc += 1;
                }
                labels.insert(queen, pc);
            }
        }

        //\\ DEBUG //\\
        for (k,v) in labels.iter() {
            println!("label: /{} => {}/", k, v);
        }
        labels
    }

    fn newline(&mut self, card: Card) -> &Ante {
        //println!("newline {}:{}", card.rank, card.suit);
        self.line = card.suit as uint;
        self
    }

    fn jump(&mut self, card: Card) -> &Ante {
        //println!("jump {}:{}", card.rank, card.suit);
        let mut suit = card.suit;
        while self.pc < self.code.len() && self.code[self.pc].rank == K && self.code[self.pc].suit == card.suit {
            suit += card.suit;
            self.pc += 1;
        }

        if self.vars[std::char::from_u32(card.suit).unwrap()] != 0 {
            let label: uint = suit as uint;
            if self.labels.contains_key(&label) {
                self.pc = self.labels[label];
            } else {
                self.exception("can't find the label...");
            }
        }
        self
    }

    fn assign(&mut self, card: Card) -> &Ante {
        //println!("assign {}:{}", card.rank, card.suit);
        let operands = self.remaining(card);
        self.expression(operands)
    }

    fn dump(&self, card: Card, as_character: bool) -> &Ante {
        //println!("dump {}:{} as character {}", card.rank, card.suit, as_character);
        let value = self.vars[std::char::from_u32(card.suit).unwrap()];
        if as_character {
            if value >= 0 as uint && value <= 255 as uint {
                print!("{}", std::char::from_u32(value as u32).unwrap());
            } else {
                self.exception(format!("character code {} is out of 0..255 range", value).as_slice());
            }
        } else {
            print!("{}", value);
        }
        self
    }

    fn remaining(&mut self, card: Card) -> Vec<Card> {
        let mut operands: Vec<Card> = vec![card];

        while self.pc < self.code.len() {
            let card = self.code[self.pc];
            if card.rank == 0 || card.rank == K || card.rank == Q || card.rank == J {
                break;
            }
            operands.push(card);
            self.pc += 1;
        }
        operands
    }

    fn expression(&mut self, operands: Vec<Card>) -> &Ante {
        let mut initial = operands[0].rank;
        let target = std::char::from_u32(operands[0].suit).unwrap();

        if initial == A {
            initial = self.vars[target] as u32;
        }

        for i in range(1, operands.len()) {
            let mut rank = operands[i].rank;
            let suit = std::char::from_u32(operands[i].suit).unwrap();

            if rank == A {
                rank = self.vars[suit] as u32;
            }
            match suit {
                '♦' => initial += rank,
                '♥' => initial *= rank,
                '♠' => initial -= rank,
                '♣' => if rank != 0 {
                            initial /= rank;
                        } else {
                            self.exception("division by zero");
                        },
                _ => continue
            }
        }

        *self.vars.get_mut(&target) = initial as uint;
        self
    }

    // NOTE: fail! got renamed to panic!
    fn exception(&self, message: &str) {
        fail!("Ante exception: {} on line {} (pc:{})\n", message, self.line, self.pc)
    }
}


fn main() {
    println!("usage: ante filename.ante");
    Ante::new("numbers.ante".as_slice()).run();
}
