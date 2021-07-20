#pragma once
#include <string>
#include <memory>
#include <functional>
#include <thread>
#include <future>
#include <Windows.h>
#include <IbWinCppLib/WinCppLib.hpp>

#include <iostream>  // For debug

namespace Everythings {
    constexpr int debug =
#ifdef IB_EVERYTHING_DEBUG
        1;
#else
        0;
#endif

    using SearchFlags = DWORD;
    struct Search {
        using T = const SearchFlags;
        static T MatchCase = 0x00000001;
        static T MatchWholeWord = 0x00000002;
        static T MatchPath = 0x00000004;
        static T Regex = 0x00000008;
        static T MatchAccents = 0x00000010;  //abandoned?
    };

    //enum class can't be used like "Request::Path | Request::Size"
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
        static Type type(RequestFlags flag) {
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
    };

    enum class Sort : DWORD {
        Default = 1,  //best performance
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

    class QueryItem {
        RequestFlags request;
        ib::Addr p;

        QueryItem(RequestFlags request, ib::Addr p) : request(request), p(p) {}
    public:
        friend class QueryResults;

        void all(std::function<void(RequestFlags flag, void* data)> f) {
            all_until([f](RequestFlags flag, void* data) { f(flag, data); return true; });
        }

        void all_until(std::function<bool(RequestFlags flag, void* data)> f) {
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

        void* get(RequestFlags flag) {
            void* data = nullptr;
            all_until([flag, &data](RequestFlags flag_, void* data_) {
                if (flag != flag_)
                    return true;
                data = data_;
                return false;
                });
            return data;
        }

        std::wstring get_str(RequestFlags flag) {
            ib::Addr data = get(flag);
            return { (const wchar_t*)(data + sizeof(DWORD)), *(DWORD*)data };
        }
        const wchar_t* get_cstr(RequestFlags flag) {
            return ib::Addr(get(flag)) + sizeof(DWORD);
        }
        size_t get_cstr_len(RequestFlags flag) {
            return *(DWORD*)get(flag);
        }
        uint64_t get_size(RequestFlags flag = Request::Size) {
            return *(uint64_t*)get(flag);
        }
        FILETIME get_date(RequestFlags flag) {
            return *(FILETIME*)get(flag);
        }
        DWORD get_dword(RequestFlags flag) {
            return *(DWORD*)get(flag);
        }
    };

    class QueryResults {
        struct EVERYTHING_IPC_LIST2 {
            DWORD totitems;  //found items
            DWORD numitems;  //available items
            DWORD offset;  //offset of the first result
            RequestFlags request_flags;  //valid request flags
            Sort sort_type;  //actual sort type
            //EVERYTHING_IPC_ITEM2 items[numitems]
            //...
        };
        struct EVERYTHING_IPC_ITEM2 {
            DWORD flags;
            DWORD data_offset;
        };

        std::shared_ptr<uint8_t[]> p;  //#TODO: async
        ib::Addr addr() {
            return p.get();
        }
        EVERYTHING_IPC_LIST2* list2() {
            return addr();
        }
        EVERYTHING_IPC_ITEM2* items() {
            return ib::Addr(list2() + 1);
        }

        QueryResults(std::shared_ptr<uint8_t[]>&& p, DWORD id)
            : p(p),
            id(id),
            found_num(list2()->totitems),
            query_num(list2()->numitems),
            request_flags(list2()->request_flags),
            sort(list2()->sort_type)
        {}
    public:
        friend class Everything;

        DWORD id;

        DWORD found_num;  // non-const because of operator=
        DWORD query_num;
        RequestFlags request_flags;
        Sort sort;

        bool empty() { return p.get() == nullptr; }
        size_t size() { return query_num; }
        size_t length() { return query_num; }

        QueryItem operator[](size_t i) {
            return { request_flags, addr() + items()[i].data_offset };
        }

        // For std::async
        QueryResults() : p(nullptr), id(0), found_num(0), query_num(0), request_flags(0), sort((Sort)0) {}
        QueryResults& operator=(const QueryResults& a) {
            p = a.p;
            id = a.id;
            found_num = a.found_num;
            query_num = a.query_num;
            request_flags = a.request_flags;
            sort = a.sort;
            return *this;
        }
    };

    class Everything {
        std::thread thread;  //message queue is bound to thread

        HWND hwnd;
        static inline const wchar_t* wnd_prop_name = L"Everything::Ev";
        static LRESULT WINAPI wndproc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
            if constexpr (debug)
                ib::DebugOStream() << "wndproc: " << hwnd << ", " << msg << ", " << wParam << ", " << lParam << std::endl;

            switch (msg) {
            case WM_COPYDATA:
            {
                //from Everything:
                //SendMessageTimeoutW(0x00000000014d11b0, WM_COPYDATA, 66754, 15723936, SMTO_ABORTIFHUNG | SMTO_BLOCK, 3000, 0x0000000000efedb8)

                COPYDATASTRUCT* copydata = (COPYDATASTRUCT*)lParam;
                //Do not assert that copydata->dwData == _EVERYTHING_COPYDATA_QUERYREPLY(0)
                //The code in Everything's SDK is wrong. copydata->dwData is replyid and can be any value.
                //ib::DebugOStream() << L"copydata->dwData: " << copydata->dwData << std::endl;

                DWORD id = (DWORD)copydata->dwData;
                auto p = std::make_shared<uint8_t[]>(copydata->cbData);
                memcpy(p.get(), copydata->lpData, copydata->cbData);
                ReplyMessage(TRUE);

                Everything* ev = (Everything*)GetPropW(hwnd, wnd_prop_name);
                if (!ev)
                    return FALSE;  //going to destruct

                if constexpr (debug) {
                    ib::DebugOStream dout;
                    dout << L"ReplyMessage" << std::endl;
                    ev->results_promise.set_value({ std::move(p), id });  dout << L"results_promise: set" << std::endl;
                    bool read = ev->results_read.get_future().get();
                    dout << L"results_read: get " << read << std::endl;
                    if (!read)
                        return TRUE;  //going to destruct, no more need to make the promise
                    ev->results_read = std::promise<bool>();  dout << L"results_read: new" << std::endl;
                    return TRUE;
                }
                ev->results_promise.set_value({ std::move(p), id });
                if(!ev->results_read.get_future().get())
                    return TRUE;  //going to destruct, no more need to make the promise
                ev->results_read = std::promise<bool>();

                return TRUE;
            }
            default:
                return DefWindowProcW(hwnd, msg, wParam, lParam);
            }
        }
        
        std::promise<QueryResults> results_promise;
        std::promise<bool> results_read;

    public:
        Everything() {
            std::promise<HWND> promise_hwnd;
            std::future<HWND> future_hwnd = promise_hwnd.get_future();

            thread = std::thread([](Everything& ev, std::promise<HWND>&& promise_hwnd) {
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
                promise_hwnd.set_value(hwnd);
                if constexpr (debug)
                    std::cout << "hwnd: " << hwnd << std::endl;
                
                SetPropW(hwnd, wnd_prop_name, &ev);

                //needed for receiving SendMessage
                MSG msg;
                DWORD ret;
                while (ret = GetMessageW(&msg, hwnd, 0, 0)) {
                    if (ret == -1)
                        break;
                
                    //won't get WM_COPYDATA here
                    if constexpr (debug)
                        ib::DebugOStream() << L"GetMessage: " << msg.message << L", " << msg.wParam << L", " << msg.lParam << std::endl;

                    switch (msg.message) {
                    case WM_APP:  //SendQuery(COPYDATASTRUCT*, 0)
                    {
                        static HWND ev_hwnd = 0;
                        if (!IsWindow(ev_hwnd))
                            ev_hwnd = FindWindowW(L"EVERYTHING_TASKBAR_NOTIFICATION", 0);

                        COPYDATASTRUCT* copydata = ib::Addr(msg.wParam);
                        if constexpr (debug) {
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
                if constexpr (debug)
                    ib::DebugOStream() << "GetMessage: break" << std::endl;
            
                }, std::ref(*this), std::move(promise_hwnd));  //#TODO

            hwnd = future_hwnd.get();
        }

        ~Everything() {
            //exit the msg loop
            PostMessageW(hwnd, WM_QUIT, 0, 0);
            //DestroyWindow(hwnd);  //doesn't work

            //exit waiting for results_read
            RemovePropW(hwnd, wnd_prop_name);
            results_read.set_value(false);
            
            //it should be safe, so needn't to join
            thread.detach();
        }

        bool query_send(const std::wstring& search, SearchFlags search_flags, RequestFlags request_flags, Sort sort = Sort::Name_Ascending, DWORD id = 0, DWORD offset = 0, DWORD max_results = 0xFFFFFFFF) {
            //Make QueryData
            struct EVERYTHING_IPC_QUERY2 {
                DWORD reply_hwnd;  //!: not sizeof(HWND)
                DWORD reply_copydata_message;
                SearchFlags search_flags;
                DWORD offset;
                DWORD max_results;
                RequestFlags request_flags;
                Sort sort_type;
                WCHAR search_string[1];  //'\0'
            };

            size_t len = search.size();
            DWORD data_len = DWORD(sizeof(EVERYTHING_IPC_QUERY2) + len * sizeof(wchar_t));
            EVERYTHING_IPC_QUERY2* data = (EVERYTHING_IPC_QUERY2*)new uint8_t[data_len];

            data->reply_hwnd = (DWORD)hwnd;
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
            copydata->dwData = 18;  //EVERYTHING_IPC_COPYDATA_QUERY2W
            copydata->lpData = data;

            //SendQuery
            
            //available: SendMessageW (blocked), SendMessageTimeoutW (unstable)
            //unavailable: PostMessageW, SendNotifyMessageW
            //not tested: SendMessageCallbackW
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

    private:
        bool query_future_first = true;
    public:
        // You must retrieve the returned future before call again
        std::future<QueryResults> query_future() {
            if constexpr (debug) {
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

        // Equals to query_future().get()
        QueryResults query_get() {
            if constexpr (debug) {
                QueryResults results = query_future().get();
                ib::DebugOStream() << L"results_promise: get" << std::endl;
                return results;
            }
            return query_future().get();
        }
    };
}