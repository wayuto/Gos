$import "io"
$import "string"

pub fun main(): int {
    # Get file name from line
    let filename: str = input("File Name: ")

    # Write file
    let fd: int = fopen(filename, O_WRONLY | O_CREAT | O_TRUNC, 420)
    let raw: str = "Hello Alum!"
    let len: int = strlen(raw)
    fwrite(fd, raw, len)
    fclose(fd)
    
    # Read file
    let fp: int = fopen(filename, O_RDONLY, 0)
    let ctx: str = fread(fp)
    println(ctx)
    fclose(fp)
    return 0
}