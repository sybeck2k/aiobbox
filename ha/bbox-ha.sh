#!/bin/bash
# Wrapper script called by Home Assistant's command_line integration.
# Store this at a path accessible to HA (e.g. /config/scripts/bbox-ha.sh)
# and make it executable: chmod +x bbox-ha.sh
#
# The BBOX_PASSWORD can be kept here so it never appears in configuration.yaml.
# Alternatively, source it from a separate secrets file:
#   source "$(dirname "$0")/bbox-secrets.env"

# BBOX_BASE_URL defaults to https://mabbox.bytel.fr/api/v1/
# Override if your router is accessed directly, e.g.:
#   BBOX_BASE_URL="http://192.168.1.254/api/v1/"
BBOX_PASSWORD="YOUR_PASSWORD_HERE" \
  /path/to/bbox --ha-json
