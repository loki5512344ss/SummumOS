char *s = "void pesc(char *t) {\n    while (*t) {\n        if (*t == 10) { putchar(92); putchar(110); }\n        else if (*t == 34) { putchar(92); putchar(34); }\n        else if (*t == 92) { putchar(92); putchar(92); }\n        else putchar(*t);\n        t = t + 1;\n    }\n}\nint main() {\n    print(\"char *s = \\\"\");\n    pesc(s);\n    print(\"\\\";\");\n    putnl();\n    print(s);\n    return 0;\n}";
void pesc(char *t) {
    while (*t) {
        if (*t == 10) { putchar(92); putchar(110); }
        else if (*t == 34) { putchar(92); putchar(34); }
        else if (*t == 92) { putchar(92); putchar(92); }
        else putchar(*t);
        t = t + 1;
    }
}
int main() {
    print("char *s = \"");
    pesc(s);
    print("\";");
    putnl();
    print(s);
    return 0;
}