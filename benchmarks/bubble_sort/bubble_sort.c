#include <stdio.h>

int main(void) {
  int x[] = {10, 9, 8, 7, 6, 5, 4, 3, 2, 1};
  int N = sizeof x / sizeof x[0];

  for (int i = 0; i < N - 1; ++i) {
    for (int j = 0; j < N - 1 - i; ++j) {
      if (x[j] > x[j + 1]) {
        int temp = x[j];
        x[j] = x[j + 1];
        x[j + 1] = temp;
      }
    }
  }

  for (int i = 0; i < N; ++i) {
    printf("%d\n", x[i]);
  }

  return 0;
}
