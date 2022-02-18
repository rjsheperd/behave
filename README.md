# Behave
A new implementation of the extended Rothermel model.

## Requirements
* [CMake](https://cmake.org/)
* [GCC](https://gcc.gnu.org/)
* [Emscripten](https://emscripten.org)

### MacOS
```bash
# Enable xcode
xcode-install --install

# Install CMake/GCC via  Homebrew (https://brew.sh/)
brew install cmake emscripten
```

## Building
```bash
# Run CMake to build
emcmake cmake -B builds
cmake --build builds
```
