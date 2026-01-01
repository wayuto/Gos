$ifndef MACRO
$import "io"
$define MACRO 1

$define BEGIN {
$define END }

$ifdef MACRO

pub fun main(): int
BEGIN
    println("Macro works!")
    return 0
END

$endif
$endif