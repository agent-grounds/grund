;;; AR-elisp: Elisp router
;; Routes requests by path.
;;
;; ## 1. Contract
;; One route per path.
;;;; A line behind four semicolons.
;;   An indented line keeps its indent.
(defun route (p) p)
