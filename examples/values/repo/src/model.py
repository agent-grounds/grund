import json
from pathlib import Path

# The comment binding is checked too: `1e-3` (§CONST-time-step.1)
values = json.loads((Path(__file__).parent.parent / "values/runtime.json").read_text())
damping = values["CONST-damping"][0]
