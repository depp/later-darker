// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "var.hpp"

#include "log.hpp"

namespace demo {

namespace var {

bool DebugContext;
bool AllocConsole;

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

// Definition for a configuration variable.
class VarDefinition {
public:
	constexpr VarDefinition(std::string_view name, bool *value)
		: mName{name}, mValue{value} {}

	std::string_view name() const { return mName; }
	bool &boolValue() const { return *mValue; }

private:
	std::string_view mName;
	bool *mValue;
};

const VarDefinition VarDefinitions[] = {
	{"DebugContext", &var::DebugContext},
	{"AllocConsole", &var::AllocConsole},
};

const VarDefinition *LookupVar(std::string_view name) {
	for (const VarDefinition &definition : VarDefinitions) {
		if (definition.name() == name) {
			return &definition;
		}
	}
	return nullptr;
}

bool ParseBool(std::string_view value) {
	if (value == "0" || value == "n" || value == "no" || value == "off" ||
	    value == "false") {
		return false;
	}
	if (value == "1" || value == "y" || value == "yes" || value == "on" ||
	    value == "true") {
		return true;
	}
	// FIXME: real error handling.
	FAIL("Invalid boolean.");
}

} // namespace

void ParseCommandArguments(int argCount, os_char **args) {
	ArgIterator iter{argCount, args};
	while (iter.HasArguments()) {
		os_string_view arg = iter.Next();
		std::size_t pos = arg.find('=');
		if (pos == os_string_view::npos) {
			// FIXME: show arg.
			FAIL("Invalid command-line argument syntax.",
			     log::Attr{"argument", arg});
		}
		std::string name = ToString(arg.substr(0, pos));
		const VarDefinition *definition = LookupVar(name);
		if (definition == nullptr) {
			// FIXME: Show full arg.
			FAIL("Command-line contains a value for an unknown variable.",
			     log::Attr{"name", name});
		}
		std::string valueStr = ToString(arg.substr(pos + 1));
		bool value = ParseBool(valueStr);
		definition->boolValue() = value;
	}
}

} // namespace demo
