package main

import (
	`bytes`
	`fmt`
	`io/ioutil`
	`math/big`
	`os`
	`regexp`
	`strings`
	`unicode/utf8`
)

type Card struct {
	rank rune
	suit rune
}

type Ante struct {
	pc     int               // Program counter (index within ante.code)
	line   int               // Current line number.
	code   []Card            // Array of cards.
	vars   map[rune]*big.Int // Four registers hashed by suit.
	labels map[rune]int      // Labels for ante.pc to jump to.
	buffer []byte            // Buffer to collect UTF-8 character bytes.
}

func (ante *Ante) Initialize() *Ante {
	ante.labels = map[rune]int{}
	ante.vars = map[rune]*big.Int{
		'♦': big.NewInt(0),
		'♥': big.NewInt(0),
		'♠': big.NewInt(0),
		'♣': big.NewInt(0),
	}

	return ante
}

func (ante *Ante) Run(filename string) *Ante {
	program, err := ioutil.ReadFile(filename)
	if err != nil {
		ante.exception(fmt.Sprintf(`%q`, err))
	}

	ante.parse(program)

	for ante.pc < len(ante.code) {
		card := ante.code[ante.pc]
		ante.pc++

		switch card.rank {
		case 0:
			ante.newline(card)
		case 'K':
			ante.jump(card)
		case 'Q':
			continue
		case 'J':
			ante.dump(card, true)
		case 10:
			ante.dump(card, false)
		default:
			ante.assign(card)
		}
	}

	return ante
}

func (ante *Ante) parse(program []byte) *Ante {
	// Split program blob into lines.
	lines := bytes.Split(program, []byte("\n"))

	// Get rid of comments and whitespaces.
	comments := regexp.MustCompile(`#.*$`)
	whitespace := regexp.MustCompile(`^\s+|\s+$`)
	for i := 0; i < len(lines); i++ {
		lines[i] = comments.ReplaceAllLiteral(lines[i], []byte(``))
		lines[i] = whitespace.ReplaceAllLiteral(lines[i], []byte(``))
		//fmt.Printf("[%s]\n", lines[i])
	}

	// Turn source file into array of cards.
	re := regexp.MustCompile(`(10|[2-9JQKA])([♦♥♠♣])`)
	for i := 0; i < len(lines); i++ {
		// Line number cards have 0 rank.
		ante.code = append(ante.code, Card{0, rune(i + 1)})
		cards := re.FindAllSubmatch(lines[i], -1)
		for _, c := range cards {
			ante.code = append(ante.code, Card{bytes.Runes(c[1])[0], bytes.Runes(c[2])[0]})
		}
	}

	// A pass to convert ranks to integers and extract labels.
	for pc := 0; pc < len(ante.code); {
		card := ante.code[pc]
		pc++
		if card.rank == '1' {
			ante.code[pc-1].rank = 10
		} else if card.rank >= '2' && card.rank <= '9' {
			ante.code[pc-1].rank = card.rank - '0'
		} else if card.rank == 'Q' {
			queen := card.suit
			for pc < len(ante.code) && ante.code[pc].rank == 'Q' && ante.code[pc].suit == card.suit {
				queen += card.suit
				pc++
			}
			ante.labels[queen] = pc
		}
	}
	//fmt.Printf("%q\n", ante.code)
	//fmt.Printf("%q\n", ante.labels)

	return ante
}

func (ante *Ante) newline(card Card) *Ante {
	ante.line++

	return ante
}

func (ante *Ante) jump(card Card) *Ante {
	//fmt.Printf("jump: %q, %d, %q\n", card, ante.pc, ante.labels)
	suit := card.suit
	for ante.pc < len(ante.code) && ante.code[ante.pc].rank == 'K' && ante.code[ante.pc].suit == card.suit {
		suit += card.suit
		ante.pc++
	}

	// if ante.vars[card.suit] != 0 ...
	if ante.vars[card.suit].Cmp(big.NewInt(0)) != 0 {
		if _, ok := ante.labels[suit]; ok {
			ante.pc = ante.labels[suit]
		} else {
			label := strings.Repeat(fmt.Sprintf(`Q%c`, card.suit), int(suit/card.suit))
			ante.exception(`can't find ` + label + ` to go`)
		}
	}

	return ante
}

func (ante *Ante) dump(card Card, char bool) *Ante {
	//fmt.Printf("dump %q\n", char)
	value := ante.vars[card.suit]
	if char {
		// if value < 0 || value > 255 ...
		if value.Cmp(big.NewInt(0)) == -1 || value.Cmp(big.NewInt(255)) == 1 {
			ante.exception(fmt.Sprintf(`character code %d is out of 0..255 range`, value))
		} else {
			// Collect the bytes till we have full UTF-8 character.
			// Once the character is built dump it and reset the buffer.
			ante.buffer = append(ante.buffer, byte(value.Int64()))
			if utf8.FullRune(ante.buffer) {
				fmt.Printf(`%s`, ante.buffer)
				ante.buffer = []byte{}
			}
		}
	} else {
		fmt.Print(value)
	}

	return ante
}

func (ante *Ante) assign(card Card) *Ante {
	//fmt.Printf("assign %q\n", card)
	operands := ante.remaining(card)

	return ante.expression(operands)
}

// Fetch the rest of the assignment expression.
func (ante *Ante) remaining(card Card) []Card {
	operands := []Card{card}
	for ante.pc < len(ante.code) {
		card = ante.code[ante.pc]
		if card.rank == 0 || card.rank == 'K' || card.rank == 'Q' || card.rank == 'J' {
			break
		}
		operands = append(operands, card)
		ante.pc++
	}

	//fmt.Printf("remaining: %q\n", operands)
	return operands
}

func (ante *Ante) expression(operands []Card) *Ante {
	initial := big.NewInt(int64(operands[0].rank))
	target := operands[0].suit

	// if initial == 'A' ...
	if initial.Cmp(big.NewInt('A')) == 0 {
		// initial = ante.vars[target] // <-- Wrong!!! We need local copy, not a pointer to register.
		initial.Set(ante.vars[target]) // <-- The right way to interpolate.
	}

	// Go through remaining operands.
	for _, card := range operands[1:] {
		rank := big.NewInt(int64(card.rank))
		suit := card.suit
		//fmt.Printf("rank: %d, suit: %c\n", rank, suit)

		// if rank == 'A' ...
		if rank.Cmp(big.NewInt('A')) == 0 {
			rank.Set(ante.vars[suit]) // <-- The right way to interpolate.
		}
		switch suit {
		case '♦':
			initial.Add(initial, rank)
		case '♥':
			initial.Mul(initial, rank)
		case '♠':
			initial.Sub(initial, rank)
		case '♣':
			if rank.Cmp(big.NewInt(0)) != 0 {
				initial.Div(initial, rank)
			} else {
				ante.exception(`division by zero`)
			}
		}
	}
	ante.vars[target] = initial
	//fmt.Printf("  %d %c => %d\n", ante.pc, target, initial)

	return ante
}

func (ante *Ante) exception(message string) {
	fmt.Printf("Ante exception: %s on line %d (pc:%d)\n", message, ante.line, ante.pc)
	os.Exit(1)
}

func main() {
	if len(os.Args) == 2 {
		new(Ante).Initialize().Run(os.Args[1])
	} else {
		fmt.Println(`usage: ante filename.ante`)
	}
}
