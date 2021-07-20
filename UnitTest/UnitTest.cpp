#include "pch.h"
#include "CppUnitTest.h"
#include "CppUnitTestLogger.h"
#include <iostream>
#include <IbEverythingLib/Everything.hpp>

using namespace Microsoft::VisualStudio::CppUnitTestFramework;
using namespace std;
using namespace Everythings;

namespace UnitTest
{
	TEST_CLASS(UnitTest)
	{
	public:
		void Query(size_t times) {
			Everything ev;
			QueryResults results;

			for (size_t i = 0; i < times; i++) {
				ev.query_send(LR"(infolder:"C:\")", 0, Request::FileName | Request::Size, Sort::Default, i);
				results = ev.query_get();
			}

			DWORD num = results.query_num;
			Logger::WriteMessage(to_wstring(num).c_str());
			for (DWORD i = 0; i < num; i++) {
				wstring s = results[i].get_str(Request::FileName);
				uint64_t size = results[i].get_size();

				wstringstream ss;
				ss << left << setw(30) << s << setw(15) << right << (size >> 10) / 1024. << L" MB" << endl;
				Logger::WriteMessage(ss.str().c_str());
			}
		}

		TEST_METHOD(TestQuery)
		{
			Query(1);
		}

		TEST_METHOD(TestQueryTwice)
		{
			Query(2);
		}

		TEST_METHOD(TestQueryTenTimes)
		{
			Query(10);
		}
	};
}