gos -c fib1000.gos
gcc -O3 fib1000.c
hyperfine -i './fib1000' './a.out' 'python fib1000.py' --shell=none --warmup 100
rm fib1000 a.out