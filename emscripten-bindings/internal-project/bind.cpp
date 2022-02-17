#include <iostream>
#include <some.h>

#include <emscripten/bind.h>

using namespace emscripten;

namespace em
{
    void myprint()
    {
        sm::lbr::printSomething();
    }

    std::vector<int> getSpreadRate()
    {
        return sm::lbr::getSpreadRate();
    }
}

EMSCRIPTEN_BINDINGS(my_module) {
    function("myprint", &em::myprint);
    function("getSpreadRate", &em::getSpreadRate);
}
