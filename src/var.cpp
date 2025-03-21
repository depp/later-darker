// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
//build: !compo
#include "var.hpp"

#include "log.hpp"

#include <optional>

namespace demo {

namespace var {

#define DEFVAR(name, type, description) Var<type> name;
#include "var_def.hpp"
#undef DEFVAR

} // namespace var

namespace {

// Iterator over command-line arguments.
class ArgIterator {
public:
	ArgIterator(int argCount, os_char **args)
		: mArg{args}, mEnd{args + argCount} {}

	// Get the next command-line argument.
	os_string_view Next() {
		if (mArg == mEnd)
			return {};
		return *mArg++;
	}

	bool HasArguments() const { return mArg != mEnd; }

private:
	os_char **mArg;
	os_char **mEnd;
};

std::optional<bool> ParseBool(std::string_view value) {
	if (value == "0" || value == "n" || value == "no" || value == "off" ||
	    value == "false") {
		return false;
	}
	if (value == "1" || value == "y" || value == "yes" || value == "on" ||
	    value == "true") {
		return true;
	}
	return std::nullopt;
}

// Kinds of variable data.
enum class Kind {
	Bool,
	String,
#if _WIN32
	WideString,
#endif
};

// Definition for a configuration variable.
class VarDefinition {
public:
	constexpr VarDefinition(std::string_view name, var::Var<bool> *value)
		: mName{name}, mKind{Kind::Bool} {
		mData.boolVar = value;
	}
	constexpr VarDefinition(std::string_view name, var::Var<std::string> *value)
		: mName{name}, mKind{Kind::String} {
		mData.stringVar = value;
	}
#if _WIN32
	constexpr VarDefinition(std::string_view name,
	                        var::Var<std::wstring> *value)
		: mName{name}, mKind{Kind::WideString} {
		mData.wideStringVar = value;
	}
#endif

	std::string_view name() const { return mName; }

	void Set(std::string_view string) const {
		switch (mKind) {
		case Kind::Bool: {
			std::optional<bool> parsed = ParseBool(string);
			if (!parsed.has_value()) {
				FAIL("Invalid boolean.", log::Attr{"var", mName},
				     log::Attr{"value", string});
			}
			mData.boolVar->set(*parsed);
		} break;
		case Kind::String:
			mData.stringVar->set(string);
			break;
		default:
			FAIL("Unsupported variable kind.");
		}
	}

#if _WIN32
	void Set(std::wstring_view string) const {
		if (mKind == Kind::WideString) {
			mData.wideStringVar->set(string);
		} else {
			Set(ToString(string));
		}
	}
#endif

private:
	std::string_view mName;
	Kind mKind;
	union {
		var::Var<bool> *boolVar;
		var::Var<std::string> *stringVar;
		var::Var<std::wstring> *wideStringVar;
	} mData;
};

const VarDefinition VarDefinitions[] = {
#define DEFVAR(name, type, description) {#name, &var::name},
#include "var_def.hpp"
#undef DEFVAR
};

const VarDefinition *LookupVar(std::string_view name) {
	for (const VarDefinition &definition : VarDefinitions) {
		if (definition.name() == name) {
			return &definition;
		}
	}
	return nullptr;
}

} // namespace

void ParseCommandArguments(int argCount, os_char **args) {
	ArgIterator iter{argCount, args};
	while (iter.HasArguments()) {
		os_string_view arg = iter.Next();
		std::size_t pos = arg.find('=');
		if (pos == os_string_view::npos) {
			FAIL("Invalid command-line argument syntax.",
			     log::Attr{"argument", arg});
		}
		std::string name = ToString(arg.substr(0, pos));
		const VarDefinition *definition = LookupVar(name);
		if (definition == nullptr) {
			FAIL("Command-line contains a value for an unknown variable.",
			     log::Attr{"name", name});
		}
		definition->Set(arg.substr(pos + 1));
	}
}

} // namespace demo
