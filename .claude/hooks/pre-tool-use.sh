#!/usr/bin/env bash
# Auto-approve claude-man commands for MANAGER orchestration
if echo "$TOOL_USE_JSON" | grep -q "claude-man"; then
  exit 0  # Approve
fi
exit 1  # Require approval
