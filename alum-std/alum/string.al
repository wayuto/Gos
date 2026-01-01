$ifndef ALUM_STRING
$define ALUM_STRING 1

extern strlen(str): int
extern strcpy(str, str): str
extern strcat(str, str): str
extern memcmp(str, str, int): int
extern memcpy(str, str, int): str
extern memset(str, int, int): str
extern bcmp(str, str, int): int

$endif