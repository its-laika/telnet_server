#!/bin/bash

if [ -z "$(docker images -q telnet_docker:latest 2> /dev/null)" ]; then
  docker build --tag telnet_docker . --file "$(dirname "$0")/Dockerfile"
fi

docker run --rm -it telnet_docker:latest "$@"