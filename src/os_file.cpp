// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "os_file.hpp"

#include "log.hpp"
#include "os_windows.hpp"
#include "var.hpp"

namespace demo {

namespace {

// Limit on maximum file size when reading files into memory.
constexpr std::size_t MaxFileSize = 64 * 1024 * 1024;

} // namespace

bool ReadFile(std::vector<unsigned char> *data, std::string_view fileName) {
	if (var::ProjectPath.empty()) {
		FAIL("Project path is not set.");
	}
	std::wstring path{var::ProjectPath};
	AppendPath(&path, fileName);
	HANDLE h = CreateFileW(path.c_str(), FILE_READ_DATA, FILE_SHARE_READ,
	                       nullptr, OPEN_EXISTING, 0, nullptr);
	if (h == INVALID_HANDLE_VALUE) {
		LOG(Error, "Could not open file.", log::Attr{"file", fileName},
		    WindowsError::GetLast());
		return false;
	}
	HandleCloser closer{h};
	BY_HANDLE_FILE_INFORMATION info;
	if (!GetFileInformationByHandle(h, &info)) {
		LOG(Error, "Could not get file information.",
		    log::Attr{"file", fileName}, WindowsError::GetLast());
		return false;
	}
	const uint64_t size64 = Pack64(info.nFileSizeHigh, info.nFileSizeLow);
	if (size64 > MaxFileSize) {
		LOG(Error, "File is too large.", log::Attr{"file", fileName},
		    log::Attr{"size", size64}, log::Attr{"maxSize", MaxFileSize});
		return false;
	}
	const std::size_t size = static_cast<std::size_t>(size64);
	data->resize(size);
	DWORD nBytes = static_cast<DWORD>(size);
	DWORD nBytesRead;
	if (!::ReadFile(h, data->data(), nBytes, &nBytesRead, nullptr)) {
		LOG(Error, "Could not read file.", log::Attr{"file", fileName},
		    WindowsError::GetLast());
		return false;
	}
	data->resize(nBytesRead);
	return true;
}

} // namespace demo
