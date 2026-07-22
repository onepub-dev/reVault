"""Keep the dynamically constructed Lua domain classes discoverable to LDoc."""

from pathlib import Path
import re
from model_docs import DESCRIPTIONS, wrapper_description

ROOT = Path(__file__).resolve().parents[1]
API = ROOT / "lua/revault_api.lua"
source = API.read_text()
source = re.sub(r"(?:--- .*\n-- @type \w+\n)?model\(\"(\w+)\"\)", lambda match: (
    f"--- {DESCRIPTIONS.get(match.group(1), wrapper_description(match.group(1)))}\n"
    f"-- @type {match.group(1)}\nmodel(\"{match.group(1)}\")"
), source)
API.write_text(source)
