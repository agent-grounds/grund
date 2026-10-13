;;; router.lisp --- the request router
;;;
;;; The routing this file applies is the one §FS-001-alpha fixes,
;;; and this file header runs four lines on purpose.

;; Returns the route for a path, following
;; the rule §FS-001-alpha states, so a caller
;; never sees an empty route, and this doc
;; comment runs four lines on purpose.
(defun route (path)
  ;; The path is trimmed before the lookup because
  ;; the rule §FS-001-alpha fixes forbids padding,
  ;; and this note runs four lines, so the cap
  ;; reports it at its opening line.
  (let ((p (string-trim " " path)))
    p))
