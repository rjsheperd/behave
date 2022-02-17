#include <iostream>
#include <some.h>

int main(int argc, char *argv[])
{
    std::cout << "base application message " << argv[1] << std::endl;
    sm::lbr::printSomething();
}
