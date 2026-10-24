# Architecture Constraints

- Students sign in with the university's single sign-on; Carrel stores no
  passwords.
- Carrel runs on virtual machines that the university's central IT provides,
  behind its load balancer. There is no public cloud account.
- It is written in Python, the language library IT maintains its other tools in.
