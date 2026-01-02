$ifndef MACRO
$define MACRO 1

$import "io"

$define BEGIN {
$define END return 0 }

$ifdef MACRO
$define MAIN pub fun main(): int
$define PRINT println

MAIN
BEGIN
    PRINT("WORKS!")
END

$endif
$endif