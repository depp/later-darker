// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "os_unix.hpp"

#include "log.hpp"

#include <cstring>

#include <errno.h>

namespace demo {

namespace {

std::string GetErrorText(int errorCode) {
	std::string str;
	str.resize(255);
	strerror_r(errorCode, str.data(), str.size());
	str.resize(std::strlen(str.data()));
	return str;
}

} // namespace

UnixError::UnixError(int errorCode)
	: mError{errorCode}, mText{GetErrorText(errorCode)} {}

void UnixError::AddToRecord(log::Record &record) const {
	if (mError != 0) {
		record.Add("error", mError);
		if (!mText.empty()) {
			record.Add("description", mText);
		}
	}
}

UnixError UnixError::Get() {
	return UnixError{errno};
}

} // namespace demo
