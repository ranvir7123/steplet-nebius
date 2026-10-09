#!/bin/bash
# S3e, in the MAIN Ubuntu: with milestone 0's signed-in Claude folder, does an API key win over the stored
# sign-in in -p mode? The key is made up and cannot work. Prints only the sign-in method and whether the run failed.
export CLAUDE_CONFIG_DIR=/home/ranvir/m0/claude-home DISABLE_AUTOUPDATER=1 CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1
C=/home/ranvir/m0/claude-cli/bin/claude
FAKE=sk-ant-api03-steplet-spike-made-up-key-0000000000
method() {
  "$C" auth status < /dev/null 2>&1 | python3 -c '
import json, sys
raw = sys.stdin.read()
try:
    print("authMethod:", json.loads(raw).get("authMethod"))
except ValueError:
    print("auth status did not print JSON; first line not shown, length", len(raw))'
}
echo "claude: $("$C" --version)"
echo "--- without a key:"; method
echo "--- with a made-up key:"; ANTHROPIC_API_KEY=$FAKE method
echo "--- one tiny prompt with the made-up key:"
ANTHROPIC_API_KEY=$FAKE "$C" -p "Reply with the single word ok" --max-turns 1 --output-format json < /dev/null 2>&1 | python3 -c '
import json, sys
raw = sys.stdin.read()
try:
    d = json.loads(raw)
    print("is_error:", d.get("is_error"), "| result:", str(d.get("result"))[:120])
except ValueError:
    print("not JSON:", raw[:200])'
