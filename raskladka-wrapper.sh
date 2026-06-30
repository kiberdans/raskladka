#!/bin/bash
ENV=$(cat /proc/$(pgrep -n plasmashell)/environ 2>/dev/null | tr '\0' '\n')
export DISPLAY=$(echo "$ENV" | grep '^DISPLAY=' | cut -d= -f2-)
export XAUTHORITY=$(echo "$ENV" | grep '^XAUTHORITY=' | cut -d= -f2-)
exec raskladka
