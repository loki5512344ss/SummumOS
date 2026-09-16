/* Hello World, но через компилятор C на Lua на Rust на эмуляторе на NAND */
int fib(int n) {
    if (n < 2) return n;
    return fib(n - 1) + fib(n - 2);
}

int counter = 100;

int main() {
    char *msg = "Hello, World!";
    int i;
    int a[5];
    int sum = 0;
    for (i = 0; msg[i] != 0; i++) putchar(msg[i]);
    putnl();
    print("fib(10) = "); print_num(fib(10)); putnl();
    for (i = 0; i < 5; i++) a[i] = i * i;
    for (i = 0; i < 5; i++) sum += a[i];
    print("sum of squares = "); print_num(sum); putnl();
    counter = counter - 142;
    print("signed: "); print_num(counter); putnl();
    if (sum > 20 && sum < 40) print("range ok"); else print("range BAD");
    putnl();
    return 0;
}
