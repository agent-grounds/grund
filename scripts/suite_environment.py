"""The environment a suite wrapper starts things with (§AR-ci-suite-environment).

Both suite wrappers, `scripts/run_cargo_suite.py` and `scripts/run_python_gate.py`, take it
from here, so what a hook says about its repository stops at the wrapper.
"""
import subprocess


def without_repository_variables(environment):
    """A copy of `environment` without the names `git rev-parse --local-env-vars` prints.

    §AR-ci-suite-environment.1: git is asked rather than a list kept, and with no `git` to
    start the copy is `environment` as it stands. A `git` that starts and fails raises, with
    its own message on stderr, because going on would hand a hook's location to every test.
    """
    copy = dict(environment)
    try:
        listed = subprocess.run(['git', 'rev-parse', '--local-env-vars'], env=copy,
                                stdout=subprocess.PIPE, text=True, check=True).stdout
    except OSError:
        return copy
    for name in listed.split():
        copy.pop(name, None)
    return copy
