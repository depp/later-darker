// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
//build: unix && !compo
#pragma once

#include <string>

#include <unistd.h>

namespace demo {
namespace log {
class Record;
}

// A Unix error code from errno.
class UnixError {
public:
	explicit UnixError(int errorCode);
	void AddToRecord(log::Record &record) const;

	static UnixError Get();

private:
	int mError;
	std::string mText;
};

// Object for cleaning up a Windows handle.
class FileCloser {
public:
	explicit FileCloser(int fd) : mFile{fd} {}
	FileCloser(const FileCloser &) = delete;
	const FileCloser &operator=(const FileCloser &) = delete;
	~FileCloser() { ::close(mFile); }

private:
	int mFile;
};

} // namespace demo
