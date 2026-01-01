$ifndef ALUM_IO
$define ALUM_IO 1

$define O_RDONLY 0
$define O_WRONLY 1
$define O_RDWR 2
$define O_CREAT 64
$define O_TRUNC 512
$define O_APPEND 1024

$define SEEK_SET 0
$define SEEK_CUR 1
$define SEEK_END 2

extern write(int, str, int): int
extern read(int, str, int): int
extern print(str): int
extern println(str): int
extern input(str): str
extern fopen(str, int, int): int
extern fclose(int): int
extern fread(int): str
extern fwrite(int, str, int): int
extern lseek(int, int, int): int

$endif