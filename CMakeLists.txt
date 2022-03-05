cmake_minimum_required(VERSION 3.5 FATAL_ERROR)

# Emscripten Compilation
set(WRAPPER_FILE ${CMAKE_CURRENT_SOURCE_DIR}/wrapper.cpp)
set(GLUE_CPP_FILE ${CMAKE_CURRENT_SOURCE_DIR}/glue.cpp)
set(GLUE_JS_FILE ${CMAKE_CURRENT_SOURCE_DIR}/glue.js)
set(BEHAVE_SRC_DIR ${CMAKE_CURRENT_SOURCE_DIR}/src/behave/)
set(BEHAVE_LIB libbehave.a)

set(EMCC_ARGS
  -I${BEHAVE_SRC_DIR}
  -I${GLUE_CPP_FILE}
  -L${BEHAVE_LIB}
  --post-js ${GLUE_JS_FILE}
  -s NO_DISABLE_EXCEPTION_CATCHING
  -s EXPORTED_FUNCTIONS=["_malloc","_free"]
  -s EXPORTED_RUNTIME_METHODS=["UTF8ToString","allocateUTF8","addFunction"])

set(EMCC_JS_ARGS ${EMCC_ARGS} -s WASM=0)

set(EMCC_WASM_ARGS ${EMCC_ARGS}
  -O2
  --closure 1
  -s BINARYEN_IGNORE_IMPLICIT_TRAPS=1
  -s WASM=1)

# Make use of c++11
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++14")

project(behave LANGUAGES CXX)

include_directories(${CMAKE_SOURCE_DIR}/src/behave)

set(SOURCE
    src/behave/behaveRun.cpp
    src/behave/behaveUnits.cpp
    src/behave/SIGContain.cpp
    src/behave/SIGContainAdapter.cpp
    src/behave/SIGContainForce.cpp
    src/behave/SIGContainForceAdapter.cpp
    src/behave/SIGContainResource.cpp
    src/behave/SIGContainSim.cpp
    src/behave/crown.cpp
    src/behave/crownInputs.cpp
    src/behave/fireSize.cpp
    src/behave/fuelModelSet.cpp
    src/behave/ignite.cpp
    src/behave/igniteInputs.cpp
    src/behave/newext.cpp
    src/behave/palmettoGallberry.cpp
    src/behave/randfuel.cpp
    src/behave/randthread.cpp
    src/behave/safety.cpp
    src/behave/SIGSpot.cpp
    src/behave/SIGSpotInputs.cpp
    src/behave/surface.cpp
    src/behave/surfaceFireReactionIntensity.cpp
    src/behave/surfaceFuelbedIntermediates.cpp
    src/behave/surfaceInputs.cpp
    src/behave/surfaceFire.cpp
    src/behave/surfaceTwoFuelModels.cpp
    src/behave/westernAspen.cpp
    src/behave/windAdjustmentFactor.cpp
    src/behave/windSpeedUtility.cpp)

set(HEADERS
    src/behave/behaveRun.h
    src/behave/behaveUnits.h
    src/behave/SIGDiurnalROS.h
    src/behave/SIGContain.h
    src/behave/SIGContainAdapter.h
    src/behave/SIGContainForce.h
    src/behave/SIGContainForceAdapter.h
    src/behave/SIGContainResource.h
    src/behave/SIGContainSim.h
    src/behave/crown.h
    src/behave/crownInputs.h
    src/behave/fireSize.h
    src/behave/fuelModelSet.h
    src/behave/ignite.h
    src/behave/igniteInputs.h
    src/behave/newext.h
    src/behave/palmettoGallberry.h
    src/behave/randfuel.h
    src/behave/randthread.h
    src/behave/safety.h
    src/behave/SIGSpot.h
    src/behave/SIGSpotInputs.h
    src/behave/surface.h
    src/behave/surfaceFireReactionIntensity.h
    src/behave/surfaceFuelbedIntermediates.h
    src/behave/surfaceInputs.h
    src/behave/surfaceFire.h
    src/behave/surfaceTwoFuelModels.h
    src/behave/westernAspen.h
    src/behave/windAdjustmentFactor.h
    src/behave/windSpeedUtility.h)

source_group("Behave Core Source Files" FILES ${SOURCE})

source_group("Behave Core Header Files" FILES ${HEADERS})

add_library(behave STATIC ${SOURCE} ${HEADERS})

add_custom_command(
  OUTPUT behave.js
  COMMAND emcc ${WRAPPER_FILE} ${EMCC_JS_ARGS} -o behave.js
  DEPENDS behave
  COMMENT "Building javascript"
  VERBATIM)
add_custom_target(behave-javascript ALL DEPENDS behave.js)

add_custom_command(
  OUTPUT behave.wasm
  COMMAND emcc ${WRAPPER_FILE} ${EMCC_WASM_ARGS} -o behave-min.js
  DEPENDS behave
  COMMENT "Building webassembly"
  VERBATIM)
add_custom_target(behave-webassembly ALL DEPENDS behave.wasm)
