#include <stdio.h>
#include <stdlib.h>

char *fizzbuzz(int n) {
    if (n % 3 == 0 && n % 5 == 0) {
        return "FizzBuzz";
    } else if (n % 3 == 0) {
        return "Fizz";
    } else if (n % 5 == 0) {
        return "Buzz";
    }
    char *result = malloc(20 * sizeof(char));
    if (result == NULL) {
        perror("malloc");
        exit(EXIT_FAILURE);
    }
    sprintf(result, "%d", n);
    return result;
}

int parse_max(int argc, char *argv[]) {
    if (argc < 2) {
        return 100; // default max
    }
    return atoi(argv[1]);
}

int perform(int max) {
    for (int i = 1; i <= max; i++) {
        printf("%d: %s\n", i, fizzbuzz(i));
    }
    return 0;
}

int main(int argc, char *argv[]) {
    int max = parse_max(argc, argv);
    perform(max);
    return 0;
}