#include <iostream>
#include <string>
#include "Everything.hpp"

#pragma comment(lib, "winmm.lib")

using namespace std;

int main()
{
    using namespace Everythings;

    DWORD t = timeGetTime();
    {
        Everything ev;
        ev.query_send(LR"(infolder:"C:\Program Files (x86)\")", 0, Request::FileName | Request::Size);
        QueryResults results = ev.query_get();  //or query_future().get()
        DWORD num = results.query_num;
        cout << num << " ";
        for (DWORD i = 0; i < num; i++) {
            wstring s = results[i].get_str(Request::FileName);
            uint64_t size = results[i].get_size();
        }
    }
    cout << timeGetTime() - t << "ms" << endl;
}