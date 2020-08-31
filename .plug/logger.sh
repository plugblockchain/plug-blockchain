# Exports logging functions and variables.

set -o errexit
set -o nounset
set -o pipefail

export TERM=${TERM:-xterm}

bold=$(tput bold)
red=$(tput setaf 1)
green=$(tput setaf 2)
reset=$(tput sgr0)

# Prints info with green foreground colour and redirect to stdout.
info() {
  printf "${bold}info: ${reset}${green}$1${reset}\n" >&1
}

# Prints error with red foreground colour and redirect to stderr.
error() {
  printf "${bold}error: ${reset}${red}$1${reset}\n" >&2
}
