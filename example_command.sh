#!/bin/sh
#
# This is an example emitter for the "command" type. In the configuration
# block, using this command must be named `mycommand`.
#

set -eou pipefail

r=$RANDOM
echo '{"name": "mycommand", "value": "'$r'", "percent": '$(($r % 100))'}'
