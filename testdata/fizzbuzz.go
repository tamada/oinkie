package main

import (
	"fmt"
	"os"
	"strconv"
)

func fizzbuzz(i int) string {
	switch {
	case i%15 == 0:
		return "FizzBuzz"
	case i%3 == 0:
		return "Fizz"
	case i%5 == 0:
		return "Buzz"
	default:
		return strconv.Itoa(i)
	}
}

func parseMax(args []string) int {
	if len(args) < 1 {
		return 100
	}
	max, err := strconv.Atoi(args[0])
	if err != nil || max < 1 {
		return 100
	}
	return max
}

func goMain(args []string) int {
	max := parseMax(args)
	for i := 1; i <= max; i++ {
		fmt.Printf("%d: %s\n", i, fizzbuzz(i))
	}
	return 0
}

func main() {
	status := goMain(os.Args[1:])
	os.Exit(status)
}
