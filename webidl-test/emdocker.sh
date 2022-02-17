#!/bin/bash

docker run -it  --rm   -v $(pwd):/src  -v /home/tdog/workspace/projects/git_projects/emscripten:/emscripten -u $(id -u):$(id -g)   emscripten/emsdk
