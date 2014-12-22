#!/usr/bin/env xcrun swift
//
// Copyright (c) 2014-2015 Michael Dvorkin
// Ante is an esoteric programming language where all you've got is a deck of cards.
//
// This is Ante implementation in Swift.

import Darwin // Posix exit()
import Foundation

typealias BigInt = Int64 // Pending arbitrary arithmetic BigInt implementation.

extension Int {
    // Returns a character from its integer ordinal.
    var chr: Character {
        return Character(UnicodeScalar(self))
    }
}

extension Character {
    // Returns integer ordinal of the character.
    var ord: UInt {
        for s in String(self).unicodeScalars {
            return UInt(s.value)
        }
        return 0
    }
}

extension String {
    subscript (i: Int) -> String {
        return String(Array(self)[i])
    }

    var size: Int {
        return countElements(self)
    }

    func split() -> Array<String> {
        return self.componentsSeparatedByString("\n")
    }

    func strip() -> String {
        return self.stringByTrimmingCharactersInSet(NSCharacterSet.whitespaceCharacterSet())
    }

    func repeat(n: Int) -> String {
        return stringByPaddingToLength(n * self.size, withString: self, startingAtIndex: 0)
    }

    func scan(pattern: String) -> [[String]] {
        var captures: [[String]] = []
        let regex = NSRegularExpression(pattern: pattern, options: nil, error: nil)
        regex!.enumerateMatchesInString(self, options: nil, range: NSMakeRange(0, self.size)) {
            (match: NSTextCheckingResult!, _, _) in
            var group: [String] = []
            for i in 1..<match!.numberOfRanges {
                group.append((self as NSString).substringWithRange(match!.rangeAtIndex(i)))
            }
            captures.append(group)
        }
        return captures
    }

    func gsub(pattern: String, substitute: String) -> String {
        return self.stringByReplacingOccurrencesOfString(pattern, withString: substitute, options: .RegularExpressionSearch)
    }
}

let ACE      = Character("A").ord
let KING     = Character("K").ord
let QUEEN    = Character("Q").ord
let JACK     = Character("J").ord
let DIAMONDS = Character("♦").ord
let HEARTS   = Character("♥").ord
let SPADES   = Character("♠").ord
let CLUBS    = Character("♣").ord

struct Card {
    var rank: UInt
    var suit: UInt
}

class Ante {
    var pc:     Int = 0                        // Program counter (index within ante.code)
    var line:   UInt = 0                       // Current line number.
    var code:   [Card] = []                    // Array of cards.
    var vars:   Dictionary<UInt, BigInt> = [:] // Four registers hashed by suit.
    var labels: Dictionary<UInt, Int> = [:]    // Labels for ante.pc to jump to.
    var buffer: [UInt8] = []                   // Buffer to collect UTF-8 character bytes.

    init() {
        self.vars[DIAMONDS] = 0
        self.vars[HEARTS] = 0
        self.vars[SPADES] = 0
        self.vars[CLUBS] = 0
    }

    func run(fileName: String) {
        var error: NSError?
        let program = NSString(contentsOfFile: fileName, encoding: NSUTF8StringEncoding, error: &error)
        if error != nil {
            self.exception("\(error!.localizedDescription)")
        }
        parse(program!)

        while self.pc < self.code.count {
            let card = self.code[self.pc]
            self.pc++
            switch card.rank {
                case 0:
                    newline(card)
                case 10:
                    dump(card, asCharacter: false)
                case JACK:
                    dump(card, asCharacter: true)
                case QUEEN:
                    continue
                case KING:
                    jump(card)
                default:
                    assign(card)
            }
        }
    }

    func parse(program: String) {
        let lines = program.split().map {
            ($0 as String).gsub("#.*$", substitute: "").strip()
        }

        // Parse lines to turn them into array of cards.
        for (i, line) in enumerate(lines) {
            // Line number cards have zero rank.
            self.code.append(Card(rank: 0, suit: i + 1))

            for caps in line.scan("(10|[2-9JQKA])([♦♥♠♣])") {
                let (rank, suit) = (Character(caps[0][0]), Character(caps[1][0]))
                switch rank {
                    case "1":
                        code.append(Card(rank: 10, suit: suit.ord))
                    case "2"..."9":
                        code.append(Card(rank: rank.ord - 48, suit: suit.ord))
                    default:
                        code.append(Card(rank: rank.ord, suit: suit.ord))
                }
            }
        }

        // Extra pass to extract labels.
        var pc = 0
        while pc < self.code.count - 1 {
            let card = self.code[pc++]
            if card.rank == QUEEN {
                var queen = card.suit
                while pc < self.code.count && self.code[pc].rank == QUEEN && self.code[pc].suit == card.suit {
                    queen += card.suit
                    pc++
                }
                self.labels[queen] = pc
            }
        }
    }

    func newline(card: Card) {
        self.line = card.suit
    }

    func dump(card: Card, asCharacter: Bool) {
        let value = self.vars[card.suit]!
        if asCharacter {
            if value >= 0 && value <= 255 {
                // Collect the bytes till we have full UTF-8 character.
                // Once the character is built dump it and reset the buffer.
                self.buffer.append(UInt8(value))
                let utf8 = NSString(bytes: self.buffer, length: self.buffer.count, encoding: NSUTF8StringEncoding)
                if utf8 != nil {
                    print(utf8!)
                    self.buffer = []
                }
            } else {
                exception("character code \(value) is not in 0..255 range")
            }
        } else {
            print(value)
        }
    }

    func jump(card: Card) {
        var suit = card.suit
        while self.pc < self.code.count && self.code[self.pc].rank == KING && self.code[self.pc].suit == card.suit {
            suit += card.suit
            self.pc++
        }

        if self.vars[card.suit] != 0 {
            if self.labels[suit] != nil {
                self.pc = self.labels[suit]!
            } else {
                let label = "Q\(Int(card.suit).chr)".repeat(Int(suit / card.suit))
                exception("can't find \(label) to go to")
            }
        }
    }

    func assign(card: Card) {
        let operands = remaining(card)
        return expression(operands)
    }

    func remaining(card: Card) -> [Card] {
        var operands = [card]
        while self.pc < self.code.count {
            let card = self.code[self.pc]
            if card.rank == 0 || card.rank == KING || card.rank == QUEEN || card.rank == JACK {
                break
            }
            operands.append(card)
            self.pc++
        }
        return operands
    }

    func expression(operands: [Card]) {
        var initial = BigInt(operands[0].rank)
        let target = operands[0].suit

        if initial == BigInt(ACE) {
            initial = self.vars[target]!
        }

        for i in 1..<operands.count {
            var rank = BigInt(operands[i].rank)
            let suit = operands[i].suit

            if rank == BigInt(ACE) {
                rank = self.vars[suit]!
            }

            // Pending arbitrary arithmetic BigInt implementation we use no
            // overflow operators to be able to trap exceptions gracefully.
            switch suit {
                case DIAMONDS:
                    initial = initial &+ rank
                case HEARTS:
                    initial = initial &* rank
                case SPADES:
                    initial = initial &- rank
                case CLUBS:
                    if rank != 0 {
                        initial = initial &/ rank
                    } else {
                        exception("division by zero")
                    }
                default:
                    continue
            }
        }
        self.vars[target] = initial
    }

    func exception(message: String) {
        println("Ante exception: \(message) on line \(self.line) (pc:\(self.pc))")
        exit(0)
    }
}

if Process.arguments.count == 2 {
    Ante().run(Process.arguments[1])
} else {
    println("usage: ante.swift filename.ante");
}
