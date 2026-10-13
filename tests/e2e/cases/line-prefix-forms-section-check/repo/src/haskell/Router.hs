module Router where

-- | AR-haskell: Haskell router
-- Routes requests, per §FS-entry.1.
--
-- ## 1. Contract
-- One route per path.
route :: String -> String
route p = p
