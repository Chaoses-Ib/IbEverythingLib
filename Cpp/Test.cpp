﻿#include <iostream>
#include <iomanip>
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
        ev.query_send(LR"(infolder:"C:\")", 0, Request::FileName | Request::Size);
        QueryResults results = ev.query_get();  //or query_future().get()
        DWORD num = results.query_num;
        wcout << num << endl;
        for (DWORD i = 0; i < num; i++) {
            wstring s = results[i].get_str(Request::FileName);
            uint64_t size = results[i].get_size();
            if constexpr (Everythings::debug)
                wcout << left << setw(30) << s << setw(15) << right << (size >> 10) / 1024. << L" MB" << endl;
        }
    }
    wcout << timeGetTime() - t << L"ms" << endl;
}