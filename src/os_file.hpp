// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
//build: !compo
#pragma once

#include <string_view>
#include <vector>

namespace demo {

// Read a file into memory.
bool ReadFile(std::vector<unsigned char> *data, std::string_view fileName);

} // namespace demo
