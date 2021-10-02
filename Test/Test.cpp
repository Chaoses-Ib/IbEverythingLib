#define BOOST_TEST_MODULE Test
#include <boost/test/unit_test.hpp>
#include <fmt/format.h>

#include <IbEverythingLib/Everything.hpp>


BOOST_AUTO_TEST_SUITE(EverythingBaseTest)

    BOOST_AUTO_TEST_CASE(GetVersion) {
		using namespace Everythings;
		Everything ev;
		Everything::Version v = ev.get_version();
		BOOST_TEST_MESSAGE(fmt::format("{}.{}.{}.{} {}", v.major, v.minor, v.revision, v.build, v.target_machine));
	    // e.g. 1.4.1.1009 2
    }

	BOOST_AUTO_TEST_CASE(GetVersion_v1_5a) {
		using namespace Everythings;
		Everything ev(L"1.5a");
		Everything::Version v = ev.get_version();
		BOOST_TEST_MESSAGE(fmt::format("{}.{}.{}.{} {}", v.major, v.minor, v.revision, v.build, v.target_machine));
	    // e.g. 1.5.0.1278 2
    }

    BOOST_AUTO_TEST_CASE(IsDatabaseLoaded) {
	    using namespace Everythings;
	    Everything ev;
	    BOOST_TEST_MESSAGE(ev.is_database_loaded());
    }

	BOOST_AUTO_TEST_CASE(Futures) {
		using namespace Everythings;
        using namespace std::chrono_literals;

		Everything ev;
	    // close Everything before testing
		BOOST_REQUIRE(!ev.is_ipc_available());

		std::future<bool> ipc_available = ev.ipc_availalbe_future();
	    while (ipc_available.wait_for(10ms) == std::future_status::timeout)
		    BOOST_TEST_MESSAGE(GetTickCount());
		BOOST_TEST_MESSAGE((ipc_available.get() ? "ipc available" : "ipc unavailable"));

		std::future<bool> database_loaded = ev.database_loaded_future();
		while (database_loaded.wait_for(10ms) == std::future_status::timeout)
			BOOST_TEST_MESSAGE(GetTickCount() << " " << ev.is_ipc_available());
		BOOST_TEST_MESSAGE((database_loaded.get() ? "database loaded" : "database not loaded"));
	}

	BOOST_AUTO_TEST_CASE(IsInfoIndexed) {
		using namespace Everythings;
		Everything ev;
		BOOST_TEST_MESSAGE(fmt::format("FileSize {}", ev.is_info_indexed(Everything::Info::FileSize)));
		BOOST_TEST_MESSAGE(fmt::format("FolderSize {}", ev.is_info_indexed(Everything::Info::FolderSize)));
		BOOST_TEST_MESSAGE(fmt::format("DateCreated {}", ev.is_info_indexed(Everything::Info::DateCreated)));
		BOOST_TEST_MESSAGE(fmt::format("DateModified {}", ev.is_info_indexed(Everything::Info::DateModified)));
		BOOST_TEST_MESSAGE(fmt::format("DateAccessed {}", ev.is_info_indexed(Everything::Info::DateAccessed)));
		BOOST_TEST_MESSAGE(fmt::format("Attributes {}", ev.is_info_indexed(Everything::Info::Attributes)));
	}

BOOST_AUTO_TEST_SUITE_END()


BOOST_AUTO_TEST_SUITE(EverythingTest)

    void query(size_t times, std::wstring_view instance_name = {}) {
	    using namespace Everythings;
	    Everything ev(instance_name);
	    QueryResults results;

	    for (size_t i = 0; i < times; i++) {
		    ev.query_send(LR"(infolder:"C:\")", 0, Request::FileName | Request::Size, Sort::Default, (DWORD)i);
		    results = ev.query_get();
	    }

	    DWORD num = results.available_num;  // or results.size()
		BOOST_TEST_MESSAGE(num);
		
	    for (DWORD i = 0; i < num; i++) {
			std::wstring_view filename = results[i].get_str(Request::FileName);
			char buf[MAX_PATH];
			buf[WideCharToMultiByte(CP_ACP, 0, filename.data(), filename.size(), buf, std::size(buf), nullptr, nullptr)] = '\0';
			BOOST_TEST_MESSAGE(fmt::format("{:30} {:>15.2f} MB", buf, (results[i].get_size() >> 10) / 1024.));
	    }
    }

    BOOST_AUTO_TEST_CASE(Query) {
		query(1);
    }

	BOOST_AUTO_TEST_CASE(Query2) {
		query(2);
	}

	BOOST_AUTO_TEST_CASE(Query10) {
		query(10);
	}

	BOOST_AUTO_TEST_CASE(Query_v1_5a) {
		query(1, L"1.5a");
	}

BOOST_AUTO_TEST_SUITE_END()


BOOST_AUTO_TEST_SUITE(EverythingMTTest)

    void query(size_t times, std::wstring_view instance_name = {}) {
	    using namespace Everythings;
	    EverythingMT ev;
	    QueryResults results;

	    for (size_t i = 0; i < times; i++) {
			// no id
			results = ev.query_send(LR"(infolder:"C:\")", 0, Request::FileName | Request::Size, Sort::Default).get();
	    }

	    DWORD num = results.available_num;  // or results.size()
		BOOST_TEST_MESSAGE(num);
		
	    for (DWORD i = 0; i < num; i++) {
			std::wstring_view filename = results[i].get_str(Request::FileName);
			char buf[MAX_PATH];
			buf[WideCharToMultiByte(CP_ACP, 0, filename.data(), filename.size(), buf, std::size(buf), nullptr, nullptr)] = '\0';
			BOOST_TEST_MESSAGE(fmt::format("{:30} {:>15.2f} MB", buf, (results[i].get_size() >> 10) / 1024.));
	    }
    }

    BOOST_AUTO_TEST_CASE(Query) {
		query(1);
    }

	BOOST_AUTO_TEST_CASE(Query2) {
		query(2);
	}

	BOOST_AUTO_TEST_CASE(Query10) {
		query(10);
	}

	BOOST_AUTO_TEST_CASE(Query_v1_5a) {
		query(1, L"1.5a");
	}

BOOST_AUTO_TEST_SUITE_END()