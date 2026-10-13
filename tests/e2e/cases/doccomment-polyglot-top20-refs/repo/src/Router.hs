module Router (route) where

-- | AR-haskell: Haskell router
--
-- ## 1. Contract
--
-- Routes requests from Haskell.
route :: [String]
route =
  -- Implements §FS-entry.1.
  -- See §FS-gone-haskell.
  [ "data §api/FS-str-haskell"
  , "first line\n\
    \data §api/FS-multi-haskell"
  ]
