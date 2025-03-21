// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
//build: !compo
#pragma once

// Standard logging, for non-compo builds.

#include <cstddef>
#include <span>
#include <string_view>
#include <type_traits>
#include <vector>

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
	WideString,
};

#if _MSC_VER
// Suppress uninitialized member variable warning. This arises from the union.
// The union is only initialized when certain tags are chosen, and the string
// type must have a default constructor so it can be put inside the union.
#pragma warning(push)
#pragma warning(disable : 26495)
#endif

// A value that can be logged as part of a log statement.
class Value {
private:
	struct String {
		const char *ptr;
		size_t size;

		constexpr String() = default;
		constexpr String(std::string_view value)
			: ptr{value.data()}, size{value.size()} {}
		constexpr operator std::string_view() const { return {ptr, size}; }
	};

	struct WideString {
		const wchar_t *ptr;
		size_t size;

		constexpr WideString() = default;
		constexpr WideString(std::wstring_view value)
			: ptr{value.data()}, size{value.size()} {}
		constexpr operator std::wstring_view() const { return {ptr, size}; }
	};

public:
	// Scalars.
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

	// Strings.
	constexpr Value(const char *value) : mKind{Kind::String} {
		mData.stringValue = std::string_view{value};
	}
	template <
		typename SV,
		std::enable_if_t<
			std::conjunction_v<
				std::is_convertible<const SV &, std::string_view>,
				std::negation<std::is_convertible<const SV &, const char *>>>,
			bool> = true>
	constexpr Value(const SV &value) : mKind{Kind::String} {
		std::string_view view = value;
		mData.stringValue = view;
	}

	constexpr Value(const wchar_t *value) : mKind{Kind::WideString} {
		mData.wideStringValue = std::wstring_view{value};
	}
	template <
		typename SV,
		std::enable_if_t<std::conjunction_v<
							 std::is_convertible<const SV &, std::wstring_view>,
							 std::negation<std::is_convertible<
								 const SV &, const wchar_t *>>>,
	                     bool> = true>
	constexpr Value(const SV &value) : mKind{Kind::WideString} {
		std::wstring_view view = value;
		mData.wideStringValue = view;
	}

	// Getters.
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
	std::wstring_view WideStringValue() const {
		return mKind == Kind::WideString
		           ? std::wstring_view(mData.wideStringValue)
		           : std::wstring_view{};
	}

private:
	Kind mKind;
	union {
		long long intValue;
		unsigned long long uintValue;
		double floatValue;
		bool boolValue;
		String stringValue;
		WideString wideStringValue;
	} mData;
};

#if _MSC_VER
#pragma warning(pop)
#endif

class Record;

template <typename T>
concept AttributeProvider = requires(const T &t, Record &r) {
	{ t.AddToRecord(r) };
};

// A key-value pair that can be part of a log message.
class Attr {
public:
	constexpr Attr() = default;
	constexpr Attr(std::string_view name, Value value)
		: mName{name}, mValue{value} {}

	std::string_view name() const { return mName; }
	const Value &value() const { return mValue; }

	inline void AddToRecord(Record &record) const;

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

	static const Location Zero;

	bool is_empty() const { return file.empty(); }
};

// A record of a log message.
class Record {
public:
	Record() : mLevel{}, mLocation{}, mMessage{}, mAttributes{} {}

	Record(Level level, Location location, std::string_view message)
		: mLevel{level}, mLocation{location}, mMessage{message} {}

	Record(Level level, Location location, std::string_view message,
	       const AttributeProvider auto &...attrs)
		: mLevel{level}, mLocation{location}, mMessage{message} {
		// Note: Above, the attrs parameter is const auto& for lifetime
		// extension, since some AttributeProvider instances own data.
		((void)attrs.AddToRecord(*this), ...);
	}

	static Record CheckFailure(Location location, std::string_view condition,
	                           std::same_as<Attr> auto... attrs) {
		return Record(Level::Error, location, "Check failed.",
		              Attr{"condition", condition}, attrs...);
	}

	Level level() const { return mLevel; }
	const Location &location() const { return mLocation; }
	std::string_view message() const { return mMessage; }
	std::span<const Attr> attributes() const { return mAttributes; }

	// Add an attribute to the record.
	void Add(std::string_view name, Value value) {
		mAttributes.emplace_back(name, value);
	}

	// Log this message.
	void Log() const;

	// Show this message and exit the program.
	[[noreturn]]
	void Fail() const;

private:
	Level mLevel;
	Location mLocation;
	std::string_view mMessage;
	std::vector<Attr> mAttributes;
};

inline void Attr::AddToRecord(Record &record) const {
	record.Add(mName, mValue);
}

/// <summary>
/// Report a memory allocation failure and exit the program. This is normally
/// called from the <see cref="FAIL_ALLOC"/> macro.
/// </summary>
/// <param name="location">Source code location where the failure
/// occurred.</param> <param name="size">Size of allocation that failed.</param>
[[noreturn]]
void FailAlloc(Location location, std::size_t size);

} // namespace log
} // namespace demo

#define LOG_LOCATION \
	::demo::log::Location { \
		__FILE__, __LINE__, __func__ \
	}
