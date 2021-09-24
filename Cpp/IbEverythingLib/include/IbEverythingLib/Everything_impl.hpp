#pragma once
#include "Everything.hpp"

namespace Everythings
{
    namespace impl
    {
        constexpr int debug =
#ifdef IB_EVERYTHING_DEBUG
            1;
#else
            0;
#endif
    }

#pragma region RequestData

#pragma warning(push)
#pragma warning(disable : 26812)  // enum class
    inline RequestData::Type RequestData::type(RequestFlags flag) {
        switch (flag) {
        case Request::FileName: return Str;
        case Request::Path: return Str;
        case Request::FullPathAndFileName: return Str;
        case Request::Extension: return Str;
        case Request::Size: return Size;
        case Request::DateCreated: return Date;
        case Request::DateModified: return Date;
        case Request::DateAccessed: return Date;
        case Request::Attributes: return Dword;
        case Request::FileListFileName: return Str;
        case Request::RunCount: return Dword;
        case Request::DateRun: return Date;
        case Request::DateRecentlyChanged: return Date;
        case Request::HighlightedFileName: return Str;
        case Request::HighlightedPath: return Str;
        case Request::HighlightedFullPathAndFileName: return Str;
        default:
            throw std::invalid_argument("Invalid request flag");
        }
    }
#pragma warning(pop)

#pragma endregion


#pragma region QueryItem

    inline QueryItem::QueryItem(RequestFlags request, ib::Addr p): request(request), p(p) {}

    inline void QueryItem::all(std::function<void(RequestFlags flag, void* data)> f) {
        all_until([f](RequestFlags flag, void* data) { f(flag, data); return true; });
    }

    inline void QueryItem::all_until(std::function<bool(RequestFlags flag, void* data)> f) {
        RequestFlags request = this->request;
        ib::Addr p = this->p;
        auto read = [request, &p, f](RequestFlags flag) {
            if (!(request & flag))
                return true;
            if (!f(flag, p))
                return false;
            switch (RequestData::type(flag)) {
            case RequestData::Str:
                p += sizeof(DWORD) + (*(DWORD*)p + 1) * sizeof(wchar_t);
                break;
            case RequestData::Size:
                p += sizeof(uint64_t);
                break;
            case RequestData::Date:
                p += sizeof(FILETIME);
                break;
            case RequestData::Dword:
                p += sizeof(DWORD);
                break;
            }
            return true;
        };

        bool s = true;
        s = s && read(Request::FileName);
        s = s && read(Request::Path);
        s = s && read(Request::FullPathAndFileName);
        s = s && read(Request::Extension);
        s = s && read(Request::Size);
        s = s && read(Request::DateCreated);
        s = s && read(Request::DateModified);
        s = s && read(Request::DateAccessed);
        s = s && read(Request::Attributes);
        s = s && read(Request::FileListFileName);
        s = s && read(Request::RunCount);
        s = s && read(Request::DateRun);
        s = s && read(Request::DateRecentlyChanged);
        s = s && read(Request::HighlightedFileName);
        s = s && read(Request::HighlightedPath);
        s = s && read(Request::HighlightedFullPathAndFileName);
    }

    inline void* QueryItem::get(RequestFlags flag) {
        void* data = nullptr;
        all_until([flag, &data](RequestFlags flag_, void* data_) {
            if (flag != flag_)
                return true;
            data = data_;
            return false;
        });
        return data;
    }

    inline std::wstring_view QueryItem::get_str(RequestFlags flag) {
        ib::Addr data = get(flag);
        return { (const wchar_t*)(data + sizeof(DWORD)), *(DWORD*)data };
    }

    inline uint64_t QueryItem::get_size(RequestFlags flag) {
        return *(uint64_t*)get(flag);
    }

    inline FILETIME QueryItem::get_date(RequestFlags flag) {
        return *(FILETIME*)get(flag);
    }

    inline DWORD QueryItem::get_dword(RequestFlags flag) {
        return *(DWORD*)get(flag);
    }

#pragma endregion


#pragma region QueryResults

    inline ib::Addr QueryResults::addr() {
        return p.get();
    }

    inline QueryResults::EVERYTHING_IPC_LIST2* QueryResults::list2() {
        return addr();
    }

    inline QueryResults::EVERYTHING_IPC_ITEM2* QueryResults::items() {
        return ib::Addr(list2() + 1);
    }

    inline QueryResults::QueryResults(DWORD id, std::shared_ptr<uint8_t[]>&& p): p(p),
        id(id),
        found_num(list2()->totitems),
        available_num(list2()->numitems),
        request_flags(list2()->request_flags),
        sort(list2()->sort_type) {}

    inline bool QueryResults::empty() const { return p.get() == nullptr; }

    inline size_t QueryResults::size() const { return available_num; }

    inline QueryItem QueryResults::operator[](size_t i) {
        return { request_flags, addr() + items()[i].data_offset };
    }

    inline QueryResults::QueryResults(): p(nullptr), id(0), found_num(0), available_num(0), request_flags(0), sort((Sort)0) {}

    inline QueryResults& QueryResults::operator=(const QueryResults& a) {
        // they have to be non-const
        p = a.p;
        id = a.id;
        found_num = a.found_num;
        available_num = a.available_num;
        request_flags = a.request_flags;
        sort = a.sort;
        return *this;
    }

#pragma endregion


#pragma region EverythingBase

    template <typename DerivedT>
    LRESULT EverythingBase<DerivedT>::wndproc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
        if constexpr (impl::debug)
            ib::DebugOStream() << "wndproc: " << hwnd << ", " << msg << ", " << wParam << ", " << lParam << std::endl;

        switch (msg) {
        case WM_COPYDATA:
        {
            // from Everything:
            // SendMessageTimeoutW(0x00000000014d11b0, WM_COPYDATA, 66754, 15723936, SMTO_ABORTIFHUNG | SMTO_BLOCK, 3000, 0x0000000000efedb8)

            COPYDATASTRUCT* copydata = (COPYDATASTRUCT*)lParam;
            // Do not assert that copydata->dwData == _EVERYTHING_COPYDATA_QUERYREPLY(0)
            // The code in Everything's SDK is wrong. copydata->dwData is replyid and can be any value.
            //ib::DebugOStream() << L"copydata->dwData: " << copydata->dwData << std::endl;

            DWORD id = (DWORD)copydata->dwData;
#if __cpp_lib_shared_ptr_arrays >= 201707L
            auto p = std::make_shared<uint8_t[]>(copydata->cbData);
#else
            std::shared_ptr<uint8_t[]> p(new uint8_t[copydata->cbData]);
#endif
            memcpy(p.get(), copydata->lpData, copydata->cbData);
            ReplyMessage(TRUE);

            DerivedT* derived_p = ib::Addr(GetPropW(hwnd, wnd_prop_name));
            if (!derived_p)
                return FALSE;  //going to destruct

            derived_p->data_arrive({ id, std::move(p) });
            return TRUE;
        }
        default:
            return DefWindowProcW(hwnd, msg, wParam, lParam);
        }
    }

    template <typename DerivedT>
    EverythingBase<DerivedT>::EverythingBase(DerivedT& derived): derived(derived) {
        std::promise<HWND> promise_hwnd;
        std::future<HWND> future_hwnd = promise_hwnd.get_future();

        thread = std::thread([](EverythingBase& ev, std::promise<HWND>&& promise_hwnd) {
            WNDCLASSEXW wndclass;
            const wchar_t* classname = L"EVERYTHING_DLL_IB";
            wndclass.cbSize = sizeof WNDCLASSEXW;
            if (!GetClassInfoExW(GetModuleHandleW(0), classname, &wndclass)) {
                wndclass = { sizeof WNDCLASSEXW };  //zero struct
                wndclass.hInstance = GetModuleHandleW(0);
                wndclass.lpfnWndProc = wndproc;
                wndclass.lpszClassName = classname;
                RegisterClassExW(&wndclass);
            }

            HWND hwnd = CreateWindowExW(0, classname, nullptr, 0, 0, 0, 0, 0, HWND_MESSAGE, 0, GetModuleHandleW(0), 0);
            SetPropW(hwnd, wnd_prop_name, &ev);
            promise_hwnd.set_value(hwnd);
            if constexpr (impl::debug)
                ib::DebugOStream() << "hwnd: " << hwnd << std::endl;

            // needed for receiving SendMessage
            MSG msg;
            DWORD ret;
            while (ret = GetMessageW(&msg, hwnd, 0, 0)) {
                if (ret == -1)
                    break;

                // won't get WM_COPYDATA here
                if constexpr (impl::debug)
                    ib::DebugOStream() << L"GetMessage: " << msg.message << L", " << msg.wParam << L", " << msg.lParam << std::endl;

                switch (msg.message) {
                case WM_APP:  // SendQuery(COPYDATASTRUCT*, 0)
                    {
                        static HWND ev_hwnd = 0;
                        if (!IsWindow(ev_hwnd))
                            ev_hwnd = FindWindowW(L"EVERYTHING_TASKBAR_NOTIFICATION", 0);

                        COPYDATASTRUCT* copydata = ib::Addr(msg.wParam);
                        if constexpr (impl::debug) {
                            ib::DebugOStream() << L"SendMessage begin" << std::endl;
                            SendMessageW(ev_hwnd, WM_COPYDATA, (WPARAM)hwnd, (LPARAM)copydata);
                            ib::DebugOStream() << L"SendMessage end" << std::endl;
                            delete copydata;
                            break;
                        }
                        SendMessageW(ev_hwnd, WM_COPYDATA, (WPARAM)hwnd, (LPARAM)copydata);
                        delete copydata;
                        break;
                    }
                }
            }
            if constexpr (impl::debug)
                ib::DebugOStream() << "GetMessage: break" << std::endl;

        }, std::ref(*this), std::move(promise_hwnd));  // #TODO

        hwnd = future_hwnd.get();
    }

    template <typename DerivedT>
    EverythingBase<DerivedT>::~EverythingBase() {
        // exit the msg loop
        PostMessageW(hwnd, WM_QUIT, 0, 0);
        //DestroyWindow(hwnd);  // doesn't work

        RemovePropW(hwnd, wnd_prop_name);
        //~derived();

        // it should be safe, so needn't to join
        thread.detach();
    }

    template <typename DerivedT>
    bool EverythingBase<DerivedT>::query_send(std::wstring_view search, SearchFlags search_flags,
        RequestFlags request_flags, Sort sort, DWORD id, DWORD offset, DWORD max_results) {
        // Make QueryData
        struct EVERYTHING_IPC_QUERY2 {
            DWORD reply_hwnd;  // !: not sizeof(HWND)
            DWORD reply_copydata_message;
            SearchFlags search_flags;
            DWORD offset;
            DWORD max_results;
            RequestFlags request_flags;
            Sort sort_type;
            WCHAR search_string[1];  // '\0'
        };

        size_t len = search.size();
        DWORD data_len = DWORD(sizeof(EVERYTHING_IPC_QUERY2) + len * sizeof(wchar_t));
        EVERYTHING_IPC_QUERY2* data = (EVERYTHING_IPC_QUERY2*)new uint8_t[data_len];

        data->reply_hwnd = *(DWORD*)&hwnd;  // (DWORD)hwnd will be warned
        data->reply_copydata_message = id;
        data->search_flags = search_flags;
        data->offset = offset;
        data->max_results = max_results;
        data->request_flags = request_flags;
        data->sort_type = sort;

        search.copy(data->search_string, len);
        data->search_string[len] = L'\0';

        COPYDATASTRUCT* copydata = new COPYDATASTRUCT;
        copydata->cbData = data_len;
        copydata->dwData = 18;  // EVERYTHING_IPC_COPYDATA_QUERY2W
        copydata->lpData = data;

        // SendQuery

        // available: SendMessageW (blocked), SendMessageTimeoutW (unstable)
        // unavailable: PostMessageW, SendNotifyMessageW
        // not tested: SendMessageCallbackW
        /*
            if constexpr (debug) {
                LRESULT result;
                DWORD error;
                if (true) {
                    result = SendMessageW(ev_hwnd, WM_COPYDATA, (WPARAM)hwnd, (LPARAM)&copydata);
                }
                else {
                    result = SendMessageTimeoutW(ev_hwnd, WM_COPYDATA, (WPARAM)hwnd, (LPARAM)&copydata, 0, 1, nullptr);
                }
                error = GetLastError();
                ib::DebugOStream() << L"SendMessage: " << result << L", " << error << std::endl;
                return result || error == ERROR_TIMEOUT;
            }
            return SendMessageTimeoutW(ev_hwnd, WM_COPYDATA, (WPARAM)hwnd, (LPARAM)&copydata, 0, 1, nullptr) || GetLastError() == ERROR_TIMEOUT;
            */
        return PostMessageW(hwnd, WM_APP, (WPARAM)copydata, 0);
    }

#pragma endregion


#pragma region Everything

    inline void Everything::data_arrive(QueryResults&& results) {
        if constexpr (impl::debug) {
            ib::DebugOStream dout;
            dout << L"ReplyMessage" << std::endl;
            results_promise.set_value(std::move(results));  dout << L"results_promise: set" << std::endl;
            bool read = results_read.get_future().get();
            dout << L"results_read: get " << read << std::endl;
            if (!read)
                return;  // going to destruct, no more need to make the promise
            results_read = std::promise<bool>();  dout << L"results_read: new" << std::endl;
            return;
        }
        results_promise.set_value(std::move(results));
        if (!results_read.get_future().get())
            return;  // going to destruct, no more need to make the promise
        results_read = std::promise<bool>();
    }

    inline Everything::Everything(): EverythingBase(*this) {}

    inline Everything::~Everything() {
        // exit waiting for results_read
        results_read.set_value(false);
    }

    inline std::future<QueryResults> Everything::query_future() {
        if constexpr (impl::debug) {
            ib::DebugOStream dout;
            if (query_future_first) {
                query_future_first = false;
                auto fut = results_promise.get_future();  dout << L"results_promise: get_future" << std::endl;
                return fut;
            }
            results_promise = std::promise<QueryResults>();  dout << L"results_promise: new" << std::endl;
            results_read.set_value(true);  dout << L"results_read: set" << std::endl;
            auto fut = results_promise.get_future();  dout << L"results_promise: get_future" << std::endl;
            return fut;
        }

        if (query_future_first) {
            query_future_first = false;
            return results_promise.get_future();
        }
        results_promise = std::promise<QueryResults>();
        results_read.set_value(true);
        return results_promise.get_future();
    }

    inline QueryResults Everything::query_get() {
        if constexpr (impl::debug) {
            QueryResults results = query_future().get();
            ib::DebugOStream() << L"results_promise: get" << std::endl;
            return results;
        }
        return query_future().get();
    }

#pragma endregion


#pragma region EverythingMT

    inline void EverythingMT::data_arrive(QueryResults&& results) {
        {
            std::lock_guard lock(mutex);
            auto it = promises.find(results.id);
            it->second.set_value(std::move(results));
            promises.erase(it);
        }
    }

    inline EverythingMT::EverythingMT(): EverythingBase(*this) {}

    inline EverythingMT::~EverythingMT() {}

    inline std::future<QueryResults> EverythingMT::query_send(std::wstring_view search, SearchFlags search_flags,
                                                              RequestFlags request_flags, Sort sort, DWORD offset, DWORD max_results) {
        std::promise<QueryResults> promise{};
        std::future<QueryResults> future = promise.get_future();
        DWORD id_this;
        {
            std::lock_guard lock(mutex);
            promises[id] = std::move(promise);
            id_this = id;
            ++id;
        }
        EverythingBase::query_send(search, search_flags, request_flags, sort, id_this, offset, max_results);
        return future;
    }

#pragma endregion

}