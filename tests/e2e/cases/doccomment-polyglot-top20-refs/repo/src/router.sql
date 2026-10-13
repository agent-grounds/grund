-- AR-sql: SQL router
--
-- ## 1. Contract
--
-- Routes requests from SQL.
CREATE VIEW route AS
-- Implements §FS-entry.1.
-- See §FS-gone-sql.
SELECT 'data §api/FS-str-sql' AS one,
       'first line
data §api/FS-multi-sql
' AS many;
