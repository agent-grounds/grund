/// AR-tokenizer: The tokenizer splits a line into tokens
///
/// Each call returns the next token of the line, §FS-parser.1.
int next_token(const char *line, int *column);

static inline int parse_line(const char *line) {
    // FS-parser: an ordinary comment declares nothing, but its citation resolves.
    // See §FS-parser.
    int column = 0;
    return next_token(line, &column);
}
