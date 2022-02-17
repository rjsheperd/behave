#!/bin/bash

docker run -it  --rm   -v $(pwd):/src   -u $(id -u):$(id -g)   emscripten/emsdk
