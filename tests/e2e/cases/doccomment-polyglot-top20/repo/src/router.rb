# AR-ruby: Ruby router
#
# ## 1. Contract
#
# Routes requests from Ruby.
def route
  # Implements §FS-entry.1.
  # See §FS-gone-ruby.
  one = "data §api/FS-str-ruby"
  many = <<~EOS
    first line
    data §api/FS-multi-ruby
  EOS
  [one, many]
end
