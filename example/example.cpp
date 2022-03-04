#include <IbEverything/Everything.hpp>
#include <iostream>

int main()
{
	using namespace Everythings;
	Everything ev{};
	QueryResults results;

	ev.query_send(LR"(infolder:"C:\")", 0, Request::FileName | Request::Size, Sort::Default);
	results = ev.query_get();

	if (!ev.is_info_indexed(Everything::Info::FolderSize))
		std::cout << "Folder size is not indexed\n";

	DWORD num = results.available_num;  // or results.size()

	for (DWORD i = 0; i < num; i++) {
		std::wstring_view filename = results[i].get_str(Request::FileName);
		char filename_ansi[MAX_PATH];
		filename_ansi[WideCharToMultiByte(CP_ACP, 0, filename.data(), filename.size(), filename_ansi, std::size(filename_ansi), nullptr, nullptr)] = '\0';
		std::cout << filename_ansi << "\t" << ((results[i].get_size() >> 10) / 1024.) << " MB\n";
	}
}