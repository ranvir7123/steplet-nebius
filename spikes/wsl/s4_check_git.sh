#!/bin/bash
# S4: after an agent run, did anything land in .git? Run outside the fence. Usage: s4_check_git.sh <repo>
R=${1:?repo}
echo "repo: $R"
echo ".git/hooks: $(ls -A "$R/.git/hooks" 2>&1 | tr '\n' ' ')"
echo "planted files in .git: $(ls -A "$R/.git" | grep -c spike_planted) ($(ls -A "$R/.git" | grep spike_planted | tr '\n' ' '))"
echo "core.hooksPath: $(git -C "$R" config --local --get core.hooksPath || echo '(not set)')"
echo "config lock left behind: $(test -e "$R/.git/config.lock" && echo yes || echo no)"
echo "files in the working copy: $(ls -A "$R" | tr '\n' ' ')"
echo "git status --short --ignored:"; git -C "$R" status --short --ignored
