# FS-parser: The parser reads one expression per line

The parser reads the input a line at a time and returns one §TERM-expression for
each line. It rejects a line it cannot read, naming the column it stopped at.

## 1. Tokens

A token is a number, an operator or a parenthesis; §AR-tokenizer finds them.
