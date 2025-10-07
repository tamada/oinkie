#include <stdio.h>
#include <stdlib.h>

int parse_max(int argc, char *argv[]) {
    if (argc == 1) 
        return 10;
    return atoi(argv[1]);
}

int perform(int max) {
    int v1 = 1, v2 = 1;
    int i;
    for (i = 0; i < max; i++) {
        if (i == 0 || i == 1) {
            printf("%5d  %d\n", i + 1, 1);
        } else {
            int v3 = v1 + v2;
            printf("%5d  %d\n", i + 1, v3);
            v1 = v2;
            v2 = v3;
        }
    }
    return 0;
}

int main(int argc, char *argv[]) {
    int max = parse_max(argc, argv);
    return perform(max);
}
