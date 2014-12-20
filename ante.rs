// Copyright (c) 2014-2015 Michael Dvorkin
// Ante is an esoteric programming language where all you've got is a deck of cards.
//
// This is Ante implementation in Rust.

extern crate regex;
extern crate num;

use std::io::File;
use std::collections::HashMap;
use std::num::Zero;
use std::num::from_int;
use std::str::{is_utf8, from_utf8};
use std::char::{from_u32};
use std::os;
use regex::Regex;
use num::bigint::BigInt;

const ACE:      uint = 'A' as uint;
const JACK:     uint = 'J' as uint;
const QUEEN:    uint = 'Q' as uint;
const KING:     uint = 'K' as uint;
const DIAMONDS: uint = '♦' as uint;
const HEARTS:   uint = '♥' as uint;
const SPADES:   uint = '♠' as uint;
const CLUBS:    uint = '♣' as uint;

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
    buffer: Vec<u8>                 // Buffer to collect UTF-8 character bytes.
}

impl Ante {
    fn new(filename: &str) -> Ante {
        let code = Ante::parse(filename);
        let labels = Ante::resolve(&code);
        let mut vars: HashMap<uint, BigInt> = HashMap::new();
        vars.insert(DIAMONDS, Ante::big(0));
        vars.insert(HEARTS, Ante::big(0));
        vars.insert(SPADES, Ante::big(0));
        vars.insert(CLUBS, Ante::big(0));

        Ante {
            pc:     0,
            line:   0,
            code:   code,
            vars:   vars,
            labels: labels,
            buffer: vec![]
        }
    }

    fn run(&mut self) -> &Ante {
        while self.pc < self.code.len() {
            let card = self.code[self.pc];
            self.pc += 1;
            match card.rank {
                0u    => self.newline(card),
                10u   => self.dump(card, false),
                JACK  => self.dump(card, true),
                QUEEN => continue,
                KING  => self.jump(card),
                _     => self.assign(card)
            };
        }
        self
    }

    // Turn source file into array of cards.
    fn parse(filename: &str) -> Vec<Card> {
        let mut file = match File::open(&Path::new(filename)) {
            // The `desc` field of `IoError` is a string that describes the error.
            Err(reason) => fail!("couldn't open {} ({})", filename, reason.desc),
            Ok(file) => file,
        };
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

            // Parse lines to turn them into array of cards.
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
            if card.rank == QUEEN {
                let mut queen = card.suit as uint;
                while pc < code.len() && code[pc].rank == QUEEN && code[pc].suit == card.suit {
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
        while self.pc < self.code.len() && self.code[self.pc].rank == KING && self.code[self.pc].suit == card.suit {
            suit += card.suit as uint;
            self.pc += 1;
        }

        if !self.vars[card.suit].is_zero() {
            let label = suit as uint;
            if self.labels.contains_key(&label) {
                self.pc = self.labels[label];
            } else {
                let label = format!("Q{:c}", from_u32(card.suit as u32).unwrap());
                self.exception(format!("can't find {} to go to", label.repeat(suit / card.suit)).as_slice());
            }
        }
        self
    }

    fn assign(&mut self, card: Card) -> &Ante {
        let operands = self.remaining(card);
        self.expression(operands)
    }

    fn dump(&mut self, card: Card, as_character: bool) -> &Ante {
        let value = self.vars[card.suit].clone();
        if as_character {
            if value < Ante::big(0) || value > Ante::big(255) {
                self.exception(format!("character code {} is not in 0..255 range", value).as_slice());
            } else {
                // Collect the bytes till we have full UTF-8 character.
                // Once the character is built dump it and reset the buffer.
                self.buffer.push(value.to_u8().unwrap());
                if is_utf8(self.buffer.as_slice()) {
                    print!("{}", from_utf8(self.buffer.as_slice()).unwrap());
                    self.buffer = vec![];
                }
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
            if card.rank == 0 || card.rank == KING || card.rank == QUEEN || card.rank == JACK {
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

        if initial.to_uint().unwrap() == ACE {
            initial = self.vars[target].clone();
        }

        for i in range(1, operands.len()) {
            let mut rank = Ante::big(operands[i].rank as int);
            let suit = operands[i].suit;

            if rank.to_uint().unwrap() == ACE {
                rank = self.vars[suit].clone();
            }

            match suit {
                DIAMONDS => { initial = initial.add(&rank) },
                HEARTS   => { initial = initial.mul(&rank) },
                SPADES   => { initial = initial.sub(&rank) },
                CLUBS    => if !rank.is_zero() {
                                initial = initial.div(&rank);
                            } else {
                                self.exception("division by zero");
                            },
                _        => continue
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
    if os::args().len() == 2 {
        Ante::new(os::args()[1].as_slice()).run();
    } else {
        println!("usage: ante filename.ante");
    }
}
