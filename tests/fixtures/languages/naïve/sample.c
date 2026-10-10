#define SCALE(x) ((x) * 2)
typedef unsigned long Counter;
struct Record { int value; };
enum Color { RED, BLUE };
union Value { int number; char letter; };
int total(int value) { return SCALE(value); }
int café(void) { return 1; }
