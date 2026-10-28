#!/usr/bin/env sh
# Symlinks this directory in as the `<name>` skill, for one agent.
#
#   ./install.sh claude            → ~/.claude/skills/
#   ./install.sh agents            → ~/.agents/skills/
#   ./install.sh claude <project>  → <project>/.claude/skills/
#   ./install.sh agents <project>  → <project>/.agents/skills/
set -eu

src=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
name=<name>
tool=${1:-}
proj=${2:-}

if [ -n "$proj" ]; then
	root=$(CDPATH= cd -- "$proj" && pwd)
else
	root=$HOME
fi

case $tool in
claude) d="$root/.claude/skills" ;;
agents) d="$root/.agents/skills" ;;
*)
	echo "usage: $0 <claude|agents> [project]" >&2
	exit 2
	;;
esac

link="$d/$name"
if [ -e "$link" ] && [ ! -L "$link" ]; then
	echo "$link exists and is not a symlink — refusing to touch it" >&2
	exit 1
fi
mkdir -p "$d"
rm -f "$link"
ln -s "$src" "$link"
echo "$link -> $src"
