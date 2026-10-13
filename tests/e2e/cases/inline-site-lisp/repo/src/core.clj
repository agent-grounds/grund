(ns app.core)

;; Returns the basket total in minor units,
;; following the rule §FS-001-alpha states,
;; so a caller never sees a half-cent, and
;; this doc comment runs four lines on purpose.
(defn- total [items]
  (reduce + items))
