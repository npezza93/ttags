module café.sample;

struct Café {
    this(int value) {}
    @safe this(this) {}
    ~this() {}
    int méthode() { return 1; }
}
interface Reader { int read(); }
class Widget : Reader { int read() { return 1; } }
union Value { int integer; double decimal; }
enum Colour { red, deprecated("old") green = 2, blue = green }
enum { anonymousFirst, anonymousSecond = anonymousFirst, int typedMember = anonymousSecond }
enum answer = 42, otherAnswer = 43;
int calculate(int value) { return value; }
auto identity(T)(T value) { return value; }
