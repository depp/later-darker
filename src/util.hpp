// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

#include <cstddef>

namespace demo {
namespace util {

// Return the next larger size for a dynamic array. Guaranteed that
// (GrowSize(x)-x) is monotonic.
constexpr std::size_t GrowSize(std::size_t size) {
	// Same as Git's alloc_nr.
	return (size + 16) * 3 / 2;
}

// Return a larger size for a dynamic array which is at least the given minimum
// size.
constexpr std::size_t GrowSizeMinimum(std::size_t size, std::size_t minimum) {
	std::size_t nextLarger = GrowSize(size);
	return nextLarger >= minimum ? nextLarger : minimum;
}

} // namespace util
} // namespace demo
