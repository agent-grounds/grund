module Router where

-- | AR-haskell: Haskell router
-- Routes requests, per §FS-entry.1.
--
-- ## 1. Contract
-- One route per path.
route :: String -> String
route p = p

data Req = Req
  { path :: String
  -- ^ AR-haskell-caret: the request path
  -- per §FS-entry.1.
  --
  -- ## 1. Contract
  -- Never empty.
  }

{-| AR-haskell-block: Haskell block router
Routes requests, per §FS-entry.1.

## 1. Contract
One route per path.
-}
route2 :: String -> String
route2 p = p
