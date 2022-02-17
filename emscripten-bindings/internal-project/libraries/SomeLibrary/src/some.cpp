#include <iostream>
#include "things.h"

namespace sm
{
    namespace lbr
    {
        std::vector<int> vector1 = {1, 2, 3, 4, 5};

        void printSomething()
        {
            std::cout << "ololo, " << someString << std::endl;
        }

        std::vector<int> getSpreadRate()
        {
            return vector1;
        }
    }
}
