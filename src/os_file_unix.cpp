// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "os_file.hpp"

#include "log.hpp"
#include "os_unix.hpp"
#include "var.hpp"

#include <fcntl.h>
#include <sys/stat.h>
#include <unistd.h>

namespace demo {

namespace {

// Limit on maximum file size when reading files into memory.
constexpr std::size_t MaxFileSize = 64 * 1024 * 1024;

} // namespace

bool ReadFile(std::vector<unsigned char> *data, std::string_view fileName) {
	if (var::ProjectPath.empty()) {
		FAIL("Project path is not set.");
	}
	std::string path{var::ProjectPath};
	AppendPath(&path, fileName);
	const int fd = ::open(path.c_str(), O_RDONLY);
	if (fd == -1) {
		LOG(Error, "Could not open file.", log::Attr{"file", fileName},
		    UnixError::Get());
		return false;
	}
	FileCloser closer{fd};
	struct stat st;
	int r = ::fstat(fd, &st);
	if (r != 0) {
		LOG(Error, "Could not get file information.",
		    log::Attr{"file", fileName}, UnixError::Get());
		return false;
	}
	const off_t osize = st.st_size;
	if (osize > static_cast<off_t>(MaxFileSize)) {
		LOG(Error, "File is too large.", log::Attr{"file", fileName},
		    log::Attr{"size", osize}, log::Attr{"maxSize", MaxFileSize});
		return false;
	}
	const std::size_t size = static_cast<std::size_t>(osize);
	data->resize(size);
	for (std::size_t pos = 0; pos < size;) {
		ssize_t amt = ::read(fd, data->data() + pos, size - pos);
		if (amt < 0) {
			LOG(Error, "Could not read file.", log::Attr{"file", fileName},
			    UnixError::Get());
			return false;
		}
		if (amt == 0) {
			LOG(Error, "File changed while reading.",
			    log::Attr{"file", fileName});
			return false;
		}
		pos += amt;
	}
	return true;
}

} // namespace demo
