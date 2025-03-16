// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

// Entry points for logging. This is modeled after Go's log/slog package.

#if COMPO

// ============================================================================
// Competition Build
// ============================================================================

#include <intrin.h>

// 7 = FAST_FAIL_FATAL_APP_EXIT
#define FAIL_IMPL() __fastfail(7)
#define LOG(level, ...) (void)0
#define CHECK(condition) (void)((!!(condition)) || (FAIL_IMPL(), 0))
#define FAIL(...) FAIL_IMPL()
#define FAIL_ALLOC(size) FAIL_IMPL()

#else

// ============================================================================
// Standard Build
// ============================================================================

#include "log_standard.hpp"

/// <summary>
/// Write a message to the log. Takes a message and an optional list of
/// attributes, such as <see cref="log::demo::Attr"/>.
/// </summary>
/// <example>
/// Log a debug message with the attribute <c>x=5</c>.
/// <code>
/// int x = 5;
/// LOG(Info, "Message.", Attr("x", x));
/// </code>
/// </example>
#define LOG(level, ...) \
	::demo::log::Record{::demo::log::Level::level, LOG_LOCATION, __VA_ARGS__} \
		.Log()

/// <summary>
/// Check that a condition is true. If not, show an error message and exit the
/// program. Attributes can be added to the message, as with <see cref="LOG"/>.
/// This behaves like assert().
/// </summary>
/// <example>
/// Check that <c>ptr</c> is not null.
/// <code>
/// void *ptr = SomeFunction();
/// CHECK(ptr != nullptr);
/// </code>
/// </example>
#define CHECK(condition) \
	(void)((!!(condition)) || \
	       (::demo::log::Record::CheckFailure(LOG_LOCATION, #condition) \
	            .Fail(), \
	        0))

/// <summary>
/// Show an error message and exit the program.
/// </summary>
/// <example>
/// Exit the program with a message about a missing file.
/// <code>
/// std::string filename = "my_file.txt";
/// FAIL("File is missing.", Attr("filename", filename));
/// </code>
/// </example>
#define FAIL(...) \
	::demo::log::Record{::demo::log::Level::Error, LOG_LOCATION, __VA_ARGS__} \
		.Fail()

/// <summary>
/// Show an error message for a memory allocation failure and exit the program.
/// </summary>
#define FAIL_ALLOC(size) ::demo::log::FailAlloc(LOG_LOCATION, size)

#endif
