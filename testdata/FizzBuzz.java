public class FizzBuzz {
    public int parseMax(String[] args) {
        if (args.length == 0) {
            return 100;
        }
        try {
            return Integer.parseInt(args[0]);
        } catch (NumberFormatException e) {
            return 100;
        }
    }

    public String fizzbuzz(int n) {
        if (n % 15 == 0) {
            return "FizzBuzz";
        } else if (n % 3 == 0) {
            return "Fizz";
        } else if (n % 5 == 0) {
            return "Buzz";
        } else {
            return Integer.toString(n);
        }
    }

    public void run(String[] args) {
        int max = parseMax(args);
        for(int i = 1; i <= max; i++) {
            System.out.printf("%d: %s%n", i, fizzbuzz(i));
        }
    }

    public static void main(String[] args) {
        new FizzBuzz().run(args);
    }
}