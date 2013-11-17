package main

import (
	`bytes`
	`fmt`
	`io/ioutil`
	`regexp`
        `os`
)

type Card struct {
	rank  rune
	suit  rune
}

type Code []Card

type Ante struct {
        pc      int
        line    int 
        code    []Card
        vars    map[rune]int
        labels  map[rune]int
}

func (ante *Ante)Initialize() *Ante {
        ante.code = make(Code, 0)
        ante.vars = map[rune]int{ '♦': 0, '♥': 0, '♠': 0, '♣': 0}
        ante.labels = map[rune]int{}

        return ante
}

func (ante *Ante)Run(filename string) *Ante {
        program, err := ioutil.ReadFile(filename)
        if err != nil {
                ante.exception(fmt.Sprintf("%q", err))
        }

        ante.parse(program)

        for ante.pc < len(ante.code) {
                card := ante.code[ante.pc]
                ante.pc++
                switch card.rank {
                case 0  : ante.newline(card)
                case 'K': ante.jump(card)
                case 'Q': continue
                case 'J': ante.dump(card, true)
                case 10 : ante.dump(card, false)
                default : ante.assign(card)
                }
        }

        return ante
}

func (ante *Ante)parse(program []byte) *Ante {
        fmt.Println("parse...")
        // Split program blob into lines.
        lines := bytes.Split(program, []byte("\n"))

        // Get rid of comments and whitespaces.
        comments := regexp.MustCompile(`#.*$`)
        whitespace := regexp.MustCompile(`^\s+|\s+$`)
        for i := 0; i < len(lines); i++ {
                lines[i] = comments.ReplaceAllLiteral(lines[i], []byte(``))
                lines[i] = whitespace.ReplaceAllLiteral(lines[i], []byte(``))
                fmt.Printf("[%s]\n", lines[i])
        }

        // Turn source file into array of cards. Each card is 2-item
        // struct of rank and suit.
        re := regexp.MustCompile(`(10|[2-9JQKA])([♦♥♠♣])`)
        for i := 0; i < len(lines); i++ {
                // Line number cards have 0 rank.
                ante.code = append(ante.code, Card{ 0, rune(i + 1) })
                cards := re.FindAllSubmatch(lines[i], -1)
                for _, c := range(cards) {
                        ante.code = append(ante.code, Card{ bytes.Runes(c[1])[0], bytes.Runes(c[2])[0]})
                }
        }
        //fmt.Printf("\n%q\n", ante.code)

        // A pass to extract labels.
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
        fmt.Printf("\n%q\n", ante.code)
        fmt.Printf("%q\n", ante.labels)
        return ante
}

func (ante *Ante)newline(card Card) *Ante {
        ante.line++
        return ante
}

// # puts "jump #{card.inspect}, pc: #{@pc.inspect}, #{@labels.inspect}"
// suit = card.suit
// while @code[@pc] && @code[@pc].rank == "K" && @code[@pc].suit == card.suit
//   suit += card.suit
//   @pc += 1
// end
// 
// if instance_variable_get("@#{suit[0]}") != 0
//   # puts "jumping to " << "Q#{suit[0]}" * suit.size
//   if @labels[suit]
//     @pc = @labels[suit]
//   else
//     exception("can't find " << "Q#{suit[0]}" * suit.size << " to go to")
//   end
// end

func (ante *Ante)jump(card Card) *Ante {
        //fmt.Printf(`jump: %q, %d, %q\n`, card, ante.pc, ante.labels)
        suit := card.suit
        for ante.pc < len(ante.code) && ante.code[ante.pc].rank == 'K' && ante.code[ante.pc].suit == card.suit {
                suit += card.suit
                ante.pc++
        }
        if ante.vars[card.suit] != 0 {
                if _, ok := ante.labels[suit]; ok {
                        ante.pc = ante.labels[suit]
                } else {
                        ante.exception(`can't find Q? to go`)
                }
        }

        return ante
}

func (ante *Ante)dump(card Card, char bool) *Ante {
        //fmt.Printf("dump %q\n", char)
        value := ante.vars[card.suit]
        if char {
                if value < 0 || value > 255 {
                        ante.exception(fmt.Sprintf(`character code %d is out of 0..255 range`, value))
                } else {
                        fmt.Printf(`%c`, value)
                }
        } else {
                fmt.Print(value)
        }
        return ante
}

func (ante *Ante)assign(card Card) *Ante {
        //fmt.Printf("assign %q\n", card)
        operands := ante.remaining(card)
        return ante.expression(operands)
}

// Fetch the rest of the assignment expression.
func (ante *Ante)remaining(card Card) []Card {
        operands := []Card{ card }
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

func (ante *Ante)expression(operands []Card) *Ante {
        initial, target := int(operands[0].rank), operands[0].suit
        //fmt.Printf("expression initial: %d (%q)\n", initial, operands[0])

        if initial == 'A' {
                initial = ante.vars[target]
        }

        for _, card := range(operands[1:]) {
                rank, suit := int(card.rank), card.suit
                //fmt.Printf("expression rank: %d, %c\n", rank, suit)
                if rank == 'A' {
                        rank = ante.vars[suit]
                        //fmt.Printf("expression rAnk: %d, %c\n", rank, suit)
                }
                switch suit {
                case '♦': initial += rank
                case '♥': initial *= rank
                case '♠': initial -= rank
                case '♣':
                        if rank != 0 {
                                initial /= rank
                        } else {
                                ante.exception(`division by zero`)
                        }
                }
                //fmt.Printf("initial: %d\n", initial)
        }
        ante.vars[target] = initial
        //fmt.Printf("expression => %c: %d\n", target, initial)
        return ante
}

func (ante *Ante)exception(message string) {
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
