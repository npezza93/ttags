module aliases;

deprecated("use Count2") alias Count = int;
alias Bytes = immutable(ubyte)[];
alias Symbol = packageName.Target;
alias Instance = Container!(int);
alias Member = Container!(int).member;
alias Template = Container;
alias ArrayOf(T) = T[];
alias First = int, Second = long;
alias Pointer = int function(int argument);
alias Delegate = int delegate(int argument);
alias Function = int(int argument);
alias Lambda = (int argument) => argument;
alias Unicode名 = int;
alias
    Multiline =
        int;
deprecated("use Count") alias int Legacy, LegacyOther;
alias packageName.Target LegacySymbol;
alias int LegacyFunction(int argument);
alias int function(int argument) LegacyPointer;
alias Container!(int) LegacyInstance;
alias FunctionTemplate(T) = T(T argument);

struct Forwarder {
    int forwarded;
    alias forwarded this;
}
struct OtherForwarder {
    int forwarded;
    alias this = forwarded;
}
template Reassign(T) {
    alias Selected = int;
    Selected = T;
}
