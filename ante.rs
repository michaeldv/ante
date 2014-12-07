// Copyright (c) 2013-2104 Michael Dvorkin
// Ante is an esoteric programming language where all you've got is a deck of cards.
//
// This is Ante implementation in Rust.

extern crate regex;
extern crate num;

use std::io::File;
use std::collections::HashMap;
use std::num::Zero;
use std::num::from_int;
use std::char::from_u32;
use regex::Regex;
use num::bigint::BigInt;

const A: uint = 'A' as uint;
const J: uint = 'J' as uint;
const Q: uint = 'Q' as uint;
const K: uint = 'K' as uint;
const D: uint = '♦' as uint;
const H: uint = '♥' as uint;
const S: uint = '♠' as uint;
const C: uint = '♣' as uint;

struct Card {
    rank: uint,
    suit: uint
}

struct Ante {
    pc:     uint,                   // Program counter (index within ante.code)
    line:   uint,                   // Current line number.
    code:   Vec<Card>,              // Array of cards.
    vars:   HashMap<uint, BigInt>,  // Four registers hashed by suit.
    labels: HashMap<uint, uint>,    // Labels for ante.pc to jump to.
}

impl Ante {
    fn new(filename: &str) -> Ante {
        let mut vars: HashMap<uint, BigInt> = HashMap::new();
        vars.insert(D, Ante::big(0));
        vars.insert(H, Ante::big(0));
        vars.insert(S, Ante::big(0));
        vars.insert(C, Ante::big(0));

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
                0u  => self.newline(card),
                K   => self.jump(card),
                Q   => continue,
                J   => self.dump(card, true),
                10u => self.dump(card, false),
                _   => self.assign(card)
            };
        }
        self
    }

    // Turn source file into array of cards.
    fn parse(filename: &str) -> Vec<Card> {
        let mut file = File::open(&Path::new(filename));
        let program = file.read_to_string().unwrap();

        // Split program blob into lines getting rid of comments and whitespaces.
        let comments = Regex::new(r"#.*$").unwrap();
        let lines: Vec<String> = program.as_slice().lines().map( |line|
            comments.replace_all(line, "").as_slice().trim().to_string()
        ).collect();

        // Turn source file into array of cards. Each card a struct of rank and suit.
        let mut code: Vec<Card> = vec![];
        let card = Regex::new(r"(10|[2-9JQKA])([♦♥♠♣])").unwrap();
        for (i, line) in lines.iter().enumerate() {
            // Line number cards have zero rank.
            code.push(Card { rank: 0, suit: i + 1 });

            // Parse cards using regural expression.
            for caps in card.captures_iter(line.as_slice()) {
                let rank = caps.at(1).char_at(0);
                let suit = caps.at(2).char_at(0);
                let card = match rank {
                   '1'       => Card { rank: 10, suit: suit as uint },
                   '2'...'9' => Card { rank: rank as uint - 48, suit: suit as uint },
                   _         => Card { rank: rank as uint, suit: suit as uint }
                };
                code.push(card);
            }
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
        labels
    }

    fn newline(&mut self, card: Card) -> &Ante {
        self.line = card.suit as uint;
        self
    }

    fn jump(&mut self, card: Card) -> &Ante {
        let mut suit = card.suit as uint;
        while self.pc < self.code.len() && self.code[self.pc].rank == K && self.code[self.pc].suit == card.suit {
            suit += card.suit as uint;
            self.pc += 1;
        }

        if !self.vars[card.suit].is_zero() {
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
        let operands = self.remaining(card);
        self.expression(operands)
    }

    fn dump(&self, card: Card, as_character: bool) -> &Ante {
        let value = self.vars[card.suit].clone();
        if as_character {
            if value < Ante::big(0) || value > Ante::big(255) {
                self.exception(format!("character code {} is out of 0..255 range", value).as_slice());
            } else {
                print!("{:1c}", from_u32(value.to_u32().unwrap()).unwrap());
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
        let mut initial: BigInt = Ante::big(operands[0].rank as int);
        let target = operands[0].suit;

        if initial.to_uint().unwrap() == A {
            initial = self.vars[target].clone();
        }

        for i in range(1, operands.len()) {
            let mut rank = Ante::big(operands[i].rank as int);
            let suit = operands[i].suit;

            if rank.to_uint().unwrap() == A {
                rank = self.vars[suit].clone();
            }

            match suit {
                D => { initial = initial.add(&rank) },
                H => { initial = initial.mul(&rank) },
                S => { initial = initial.sub(&rank) },
                C => if !rank.is_zero() {
                        initial = initial.div(&rank);
                     } else {
                        self.exception("division by zero");
                     },
                _ => continue
            }
        }

        *self.vars.get_mut(&target) = initial;
        self
    }

    fn big(n: int) -> BigInt {
        from_int::<BigInt>(n).unwrap()
    }

    // NOTE: fail! got renamed to panic!
    fn exception(&self, message: &str) {
        fail!("Ante exception: {} on line {} (pc:{})\n", message, self.line, self.pc)
    }
}


fn main() {
    println!("usage: ante filename.ante");
    Ante::new("fizzbuzz.ante".as_slice()).run();
}
