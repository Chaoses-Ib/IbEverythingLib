#include <iostream>
#include <iomanip>
#include <string>
#include "IbEverythingLib/include/IbEverythingLib/Everything.hpp"

#pragma comment(lib, "winmm.lib")

using namespace std;

int main()
{
    using namespace Everythings;

    DWORD t = timeGetTime();
    {
        Everything ev;
        ev.query_send(LR"(infolder:"C:\")", 0, Request::FileName | Request::Size);
        QueryResults results = ev.query_get();  // or query_future().get()
        DWORD num = results.available_num;  // or results.size()
        wcout << num << endl;
        for (DWORD i = 0; i < num; i++) {
            wstring_view s = results[i].get_str(Request::FileName);
            uint64_t size = results[i].get_size();
            if constexpr (impl::debug)
                wcout << left << setw(30) << s << setw(15) << right << (size >> 10) / 1024. << L" MB" << endl;
        }
    }
    wcout << timeGetTime() - t << L"ms" << endl;
}