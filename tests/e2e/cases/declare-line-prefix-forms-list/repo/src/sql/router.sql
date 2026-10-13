-- AR-sql: SQL router
-- Routes requests, per §FS-entry.1.
--
-- ## 1. Contract
-- One route per path.
CREATE FUNCTION route() RETURNS int AS $$ SELECT 1 $$ LANGUAGE sql;
