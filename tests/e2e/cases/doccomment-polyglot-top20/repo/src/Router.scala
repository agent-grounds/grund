/** AR-scala: Scala router
  *
  * ## 1. Contract
  *
  * Routes requests from Scala.
  */
final class Router {
  // Implements §FS-entry.1.
  // See §FS-gone-scala.
  val one = "data §api/FS-str-scala"
  val many = """first line
    |data §api/FS-multi-scala
    |""".stripMargin
}
