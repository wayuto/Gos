$import "lib"
$import "convert"

# Extern C functions
extern add(int int): int
extern fadd(flt flt): flt

pub fun main(): int {
    println(
        itoa(
            add(1 2)
        )
    )
    println(
        ftoa(
            fadd(1.5 2.5)
        )
    )
    return 0
}