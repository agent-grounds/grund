#include "router.hpp"
// Implements §FS-entry.1.
// See §FS-gone-cpp.
static const char *one = "data §api/FS-str-cpp";
static const char *many = R"(first line
data §api/FS-multi-cpp
)";
int route() { return 0; }
