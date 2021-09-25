#pragma once
#include <string_view>
#include <memory>
#include <map>
#include <functional>
#include <thread>
#include <mutex>
#include <future>
#include <Windows.h>
#include <IbWinCppLib/WinCppLib.hpp>

namespace Everythings {

    using SearchFlags = DWORD;
    struct Search {
        using T = const SearchFlags;
        static T MatchCase = 0x00000001;
        static T MatchWholeWord = 0x00000002;
        static T MatchPath = 0x00000004;
        static T Regex = 0x00000008;
        static T MatchAccents = 0x00000010;  // abandoned?
    };

    enum class Sort : DWORD {
        Default = 1,  // best performance
        Name_Ascending = 1,
        Name_Descending = 2,
        Path_Ascending = 3,
        Path_Descending = 4,
        Size_Ascending = 5,
        Size_Descending = 6,
        Extension_Ascending = 7,
        Extension_Descending = 8,
        TypeName_Ascending = 9,
        TypeName_Descending = 10,
        DateCreated_Ascending = 11,
        DateCreated_Descending = 12,
        DateModified_Ascending = 13,
        DateModified_Descending = 14,
        Attributes_Ascending = 15,
        Attributes_Descending = 16,
        FileListFilename_Ascending = 17,
        FileListFilename_Descending = 18,
        RunCount_Ascending = 19,
        RunCount_Descending = 20,
        DateRecentlyChanged_Ascending = 21,
        DateRecentlyChanged_Descending = 22,
        DateAccessed_Ascending = 23,
        DateAccessed_Descending = 24,
        DateRun_Ascending = 25,
        DateRun_Descending = 26
    };

    // enum class cannot be used like "Request::Path | Request::Size"
    using RequestFlags = DWORD;
    struct Request {
        using T = const RequestFlags;
        static T FileName = 0x00000001;
        static T Path = 0x00000002;
        static T FullPathAndFileName = 0x00000004;
        static T Extension = 0x00000008;
        static T Size = 0x00000010;
        static T DateCreated = 0x00000020;
        static T DateModified = 0x00000040;
        static T DateAccessed = 0x00000080;
        static T Attributes = 0x00000100;
        static T FileListFileName = 0x00000200;
        static T RunCount = 0x00000400;
        static T DateRun = 0x00000800;
        static T DateRecentlyChanged = 0x00001000;
        static T HighlightedFileName = 0x00002000;
        static T HighlightedPath = 0x00004000;
        static T HighlightedFullPathAndFileName = 0x00008000;
    };


    class RequestData {
    public:
        enum Type { Str, Size, Date, Dword };

        // std::invalid_argument
        static Type type(RequestFlags flag);
    };

    class QueryItem {
    public:
        friend class QueryResults;

        void all(std::function<void(RequestFlags flag, void* data)> f);
        void all_until(std::function<bool(RequestFlags flag, void* data)> f);

        void* get(RequestFlags flag);
        std::wstring_view get_str(RequestFlags flag);
        uint64_t get_size(RequestFlags flag = Request::Size);
        FILETIME get_date(RequestFlags flag);
        DWORD get_dword(RequestFlags flag);

    protected:
        RequestFlags request;
        ib::Addr p;

        QueryItem(RequestFlags request, ib::Addr p);
    };

    class QueryResults {
    protected:
        // must be declared before other members relying on list2()
        std::shared_ptr<uint8_t[]> p;  //#TODO: async
    public:
        DWORD id;

        DWORD found_num;  // The number of found items
        DWORD available_num;  // The number of available items
        RequestFlags request_flags;  // Valid request flags
        Sort sort;  // Maybe different to requested sort type

        bool empty() const;

        // The number of available items (this->available_num)
        size_t size() const;

        // Do not release QueryResults during the use of QueryItem
        QueryItem operator[](size_t i);

        // For std::async
        QueryResults();
        QueryResults& operator=(const QueryResults& a);

    protected:
        template <typename DerivedT>
        friend class EverythingBase;

        struct EVERYTHING_IPC_LIST2 {
            DWORD totitems;  // found items
            DWORD numitems;  // available items
            DWORD offset;  // offset of the first result
            RequestFlags request_flags;  // valid request flags
            Sort sort_type;  // actual sort type
            // EVERYTHING_IPC_ITEM2 items[numitems]
            // ...
        };
        struct EVERYTHING_IPC_ITEM2 {
            DWORD flags;
            DWORD data_offset;
        };

        ib::Addr addr();
        EVERYTHING_IPC_LIST2* list2();
        EVERYTHING_IPC_ITEM2* items();

        QueryResults(DWORD id, std::shared_ptr<uint8_t[]>&& p);
    };


    template <typename DerivedT>
    class EverythingBase {
    public:
        static bool is_ipc_available();
        std::future<bool> ipc_availalbe_future();

        enum class TargetMachine : uint32_t {
            x86 = 1,
            x64 = 2,
            Arm = 3
        };
        struct Version {
            uint32_t major;
            uint32_t minor;
            uint32_t revision;
            uint32_t build;
            TargetMachine target_machine;
        };
        Version get_version() const;

        bool is_database_loaded() const;
        std::future<bool> database_loaded_future() const;

        enum class Info {
            FileSize = 1,
            FolderSize = 2,
            DateCreated = 3,
            DateModified = 4,
            DateAccessed = 5,
            Attributes = 6
        };
        bool is_info_indexed(Info info) const;

    protected:
        EverythingBase(DerivedT& derived);
        ~EverythingBase();

        bool query_send(std::wstring_view search, SearchFlags search_flags, RequestFlags request_flags, Sort sort = Sort::Name_Ascending, DWORD id = 0, DWORD offset = 0, DWORD max_results = 0xFFFFFFFF);
    
        DerivedT& derived;

        static inline HWND ipc_window;
        static void update_ipc_window();
        HANDLE ipc_event = nullptr;  // #TODO: std::shared_ptr
        uint32_t send_ipc_dword(uint32_t command, uintptr_t param = 0) const;

        std::thread thread;  // message queue is bound to thread

        HWND hwnd;
        static inline const wchar_t* wnd_prop_name = L"IbEverythingLib::EverythingBase";  // lambda capture

        static LRESULT WINAPI wndproc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam);
    };

    /*
    class EverythingBaseDerived : public EverythingBase<EverythingBaseDerived> {
    public:
        EverythingBaseDerived() : EverythingBase(*this) {}
        
        ~EverythingBaseDerived() {
            // stop data_arrive
        }

    protected:
        friend class EverythingBase<EverythingBaseDerived>;

        void data_arrive(QueryResults&& results);
    };
    */


    class Everything : public EverythingBase<Everything> {
    public:
        Everything();
        ~Everything();

        using EverythingBase::query_send;

        // You must retrieve the returned future before call again.
        // If the current results are not retrieved, the new results will be discarded after 3 seconds.
        std::future<QueryResults> query_future();

        // Equals to query_future().get()
        QueryResults query_get();

    protected:
        friend class EverythingBase<Everything>;

        std::promise<QueryResults> results_promise;
        std::promise<bool> results_read;

        void data_arrive(QueryResults&& results);
        bool query_future_first = true;
    };


    // Thread-safe
    class EverythingMT : public EverythingBase<EverythingMT> {
    public:
        EverythingMT();
        ~EverythingMT();

        std::future<QueryResults> query_send(std::wstring_view search, SearchFlags search_flags, RequestFlags request_flags, Sort sort = Sort::Name_Ascending, DWORD offset = 0, DWORD max_results = 0xFFFFFFFF);

    protected:
        friend class EverythingBase<EverythingMT>;

        DWORD id = 0;
        std::map<DWORD, std::promise<QueryResults>> promises;
        std::mutex mutex;

        void data_arrive(QueryResults&& results);
    };
}

#include "Everything_impl.hpp"