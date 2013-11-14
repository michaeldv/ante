#!/usr/bin/env ruby
# encoding: utf-8
#
# Copyright (c) 2013 Michael Dvorkin
#
# Ante is an esoteric programming language where all you've got is
# a deck of cards.
#
# 95% of this code was developed on the way back from RubyConf 2013
# during 5-hour flight from Miami to San Francisco.
# 
### require "awesome_print"

class Array
  def rank; self[0] end
  def suit; self[1] end
  def rank=(value); self[0] = value end
  def suit=(value); self[1] = value end
end

class Ante
  def initialize()
    @♦, @♥, @♠, @♣ = 0, 0, 0, 0
    @line, @pc = 0, 0
    @code, @labels = [], {}
  end

  def run(source)
    lines = source.split("\n").map { |line| line.sub(/#.*$/, "").strip }
    ### ap lines

    # Turn source file into array of cards. Each card is 2-item
    # array of rank and suit.
    lines.each_with_index do |line, i|
      @code += [[ nil, i + 1 ]] # <-- Line number cards have nil rank.
      @code += line.scan(/(10|[2-9JQKA])([♦♥♠♣])/)
    end

    # Collect labels and convert ranks from String to Fixnum.
    @code.each_with_index do |card, i|
      if card.rank =~ /\d/
        card.rank = card.rank.to_i
      elsif card.rank == "Q" 
        @labels[card.suit] = i + 1
      end
    end
    ### ap @code
    ### ap @labels

    while card = @code[@pc]
      @pc += 1
      case card.rank
      when nil then newline(card)
      when "K" then jump(card)
      when "Q" then next
      when "J" then dump(card, :char)
      when 10  then dump(card)
      else          assign(card)
      end
    end
  end

  def newline(card)
    # puts "newline #{card}"
    @line = card.suit
  end

  def assign(card)
    # puts "assign #{card.inspect}"
    operands = remaining(card)
    expression(operands)
  end

  def jump(card)
    # puts "jump #{card.inspect}"
    suit = card.suit
    if @labels[suit] && instance_variable_get("@#{suit}") != 0
      @pc = @labels[suit]
    end
  end

  def dump(card, char = nil)
    # puts "dump #{card.inspect} => "
    value = instance_variable_get("@#{card.suit}")
    print char ? value.chr : value
  end

  # Fetch the rest of the assignment expression.
  def remaining(card)
    operands = [ card ]
    while card = @code[@pc]
      break if card.rank.nil? || card.rank =~ /[KQJ]/
      operands += [ card ]
      @pc += 1
    end
    ### ap "remaining: #{operands.inspect}"
    operands
  end

  def expression(operands)
    initial, target = operands.shift
    initial = instance_variable_get("@#{target}") if initial == "A"
    operands.each do |rank, suit|
      # puts "rank: #{rank.inspect}, suit: #{suit.inspect}"
      rank = instance_variable_get("@#{suit}") if rank == "A"
      case suit
      when "♦" then initial += rank
      when "♥" then initial *= rank
      when "♠" then initial -= rank
      when "♣" then initial /= rank rescue 0
      end
    end
    instance_variable_set("@#{target}", initial)
    # dump_registers
  end

  def dump_registers
    instance_variables.each do |i|
      puts "  #{i}: " + instance_variable_get("#{i}").to_s if i.size == 2
    end
  end
end

if ARGV[0]
  Ante.new.run(IO.read(ARGV[0]))
else
  puts "usage: ante filename.ante"
end
