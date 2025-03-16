// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

#include "os_string.hpp"

#include <string>
#include <string_view>

namespace demo {

namespace var {

#if COMPO

// Variable traits, which describe operations on variables of the given type.
template <typename T>
struct VarTraits {
	using Value = T;
};

// A string variable becomes a string view constant.
template <typename T>
struct VarTraits<std::basic_string<T>> {
	using Value = std::basic_string_view<T>;
};

// Configurable variable. In competition builds, the variable is a compile-time
// constant.
template <typename T>
struct Var {
	using Traits = VarTraits<T>;
	using Value = typename VarTraits<T>::Value;
	constexpr T get() { return T{}; }
};

#define DEFVAR(name, type, description) constexpr Var<type> name;

#else

// Variable traits, which describe operations on variables of the given type.
template <typename T>
struct VarTraits {
	using Storage = T;
	using Value = T;
};

// A string variable is accessed through a string view.
template <typename T>
struct VarTraits<std::basic_string<T>> {
	using Storage = std::basic_string<T>;
	using Value = std::basic_string_view<T>;
};

// Configurable variable.
template <typename T>
class Var {
public:
	using Traits = typename VarTraits<T>;
	using Storage = typename Traits::Storage;
	using Value = typename Traits::Value;

	Value get() const { return mStorage; }
	void set(Value value) { mStorage = value; }

private:
	Storage mStorage;
};

#define DEFVAR(name, type, description) extern Var<type> name;

#endif

#include "var_def.hpp"

#undef DEFVAR

} // namespace var

#if !COMPO

// Parse the program's command-line arguments.
void ParseCommandArguments(int argCount, os_char **args);

#endif

} // namespace demo
