// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

// Logging. This is modeled after Go's log/slog package.

#include <cstddef>
#include <initializer_list>
#include <string_view>

namespace demo {
namespace log {

// Log message severity level.
enum class Level {
	Debug,
	Info,
	Warn,
	Error,
};

// A kind of value that can be logged.
enum class Kind {
	Null,
	Int,
	Uint,
	Float,
	Bool,
	String,
};

// Suppress uninitialized member variable warning. This arises from the union.
// The union is only initialized when certain tags are chosen, and the string
// type must have a default constructor so it can be put inside the union.
#pragma warning(push)
#pragma warning(disable : 26495)

// A value that can be logged as part of a log statement.
class Value {
private:
	struct String {
		const char *ptr;
		size_t size;

		constexpr String() = default;
		constexpr String(std::string_view value)
			: ptr{value.data()}, size{value.size()} {}
		constexpr operator std::string_view() const {
			return std::string_view{ptr, size};
		}
	};

public:
	constexpr Value() : mKind{Kind::Null} {}
	constexpr Value(std::nullptr_t) : mKind{Kind::Null} {}
	constexpr Value(int value) : mKind{Kind::Int} { mData.intValue = value; }
	constexpr Value(unsigned value) : mKind{Kind::Uint} {
		mData.uintValue = value;
	}
	constexpr Value(long value) : mKind{Kind::Int} { mData.intValue = value; }
	constexpr Value(unsigned long value) : mKind{Kind::Uint} {
		mData.uintValue = value;
	}
	constexpr Value(long long value) : mKind{Kind::Int} {
		mData.intValue = value;
	}
	constexpr Value(unsigned long long value) : mKind{Kind::Uint} {
		mData.uintValue = value;
	}
	constexpr Value(double value) : mKind{Kind::Float} {
		mData.floatValue = value;
	}
	constexpr Value(bool value) : mKind{Kind::Bool} { mData.boolValue = value; }
	constexpr Value(std::string_view value) : mKind{Kind::String} {
		mData.stringValue = value;
	}
	constexpr Value(const char *value) : mKind{Kind::String} {
		mData.stringValue = std::string_view{value};
	}

	Kind ValueKind() const { return mKind; }
	long long IntValue() const {
		return mKind == Kind::Int ? mData.intValue : 0ll;
	}
	unsigned long long UintValue() const {
		return mKind == Kind::Uint ? mData.uintValue : 0ull;
	}
	double FloatValue() const {
		return mKind == Kind::Float ? mData.floatValue : 0.0;
	}
	bool BoolValue() const {
		return mKind == Kind::Bool ? mData.boolValue : false;
	}
	std::string_view StringValue() const {
		return mKind == Kind::String ? std::string_view(mData.stringValue)
		                             : std::string_view{};
	}

private:
	Kind mKind;
	union {
		long long intValue;
		unsigned long long uintValue;
		double floatValue;
		bool boolValue;
		String stringValue;
	} mData;
};

#pragma warning(pop)

// A key-value pair that can be part of a log message.
class Attr {
public:
	constexpr Attr(std::string_view name, Value value)
		: mName{name}, mValue{value} {}

	std::string_view name() const { return mName; }
	const Value &value() const { return mValue; }

private:
	std::string_view mName;
	Value mValue;
};

// Initialize the logging system.
void Init();

// A location in the source code.
struct Location {
	std::string_view file;
	int line;
	std::string_view function;
};

// Write a message to the log.
void Log(Level level, const Location &location, std::string_view message);

void Log(Level level, const Location &location, std::string_view message,
         std::initializer_list<Attr> attributes);

[[noreturn]]
void CheckFail(const Location &location, std::string_view condition);

} // namespace log
} // namespace demo

#define LOG_LOCATION \
	::demo::log::Location { \
		__FILE__, __LINE__, __func__ \
	}

// Write a message to the log.
#define LOG(level, ...) \
	::demo::log::Log(::demo::log::Level::level, LOG_LOCATION, __VA_ARGS__)

// Check that a condition is true. If not, show an error message and exit the
// program.
#define CHECK(condition) \
	(void)((!!(condition)) || \
	       (::demo::log::CheckFail(LOG_LOCATION, #condition), 0))
