package router

// AR-go: Go router
//
// ## 1. Contract
//
// Routes requests from Go.
func Route() []string {
	// Implements §FS-entry.1.
	// See §FS-gone-go.
	one := "data §api/FS-str-go"
	many := `first line
data §api/FS-multi-go
`
	return []string{one, many}
}
