x = [10, 9, 8, 7, 6, 5, 4, 3, 2, 1]
N = len(x)

for i in range(N - 1):
    for j in range(N - 1 - i):
        if x[j] > x[j + 1]:
            x[j], x[j + 1] = x[j + 1], x[j]

for v in x:
    print(v)
