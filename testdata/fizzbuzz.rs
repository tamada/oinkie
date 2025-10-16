fn parse_args(args: Vec<String>) -> u32 {
    if args.len() < 2 {
        100
    } else {
        args[1].parse().unwrap_or(100)
    }
}

fn fizzbuzz(i: u32) -> String {
    match (i % 3, i % 5) {
        (0, 0) => "FizzBuzz".to_string(),
        (0, _) => "Fizz".to_string(),
        (_, 0) => "Buzz".to_string(),
        _ => i.to_string(),
    }
}

fn perform(max: u32) {
    for i in 1..=max {
        println!("{i}, {}", fizzbuzz(i));
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let max = parse_args(args);
    perform(max);
}