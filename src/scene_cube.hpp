// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once
#include "gl.hpp"

namespace demo {
namespace scene {

class Cube {
public:
	Cube() : mArray{0}, mBuffer{0} {}
	Cube(const Cube &) = delete;
	Cube &operator=(const Cube &) = delete;

	void Init();
	void Render(double time);

private:
	GLuint mArray;
	GLuint mBuffer[2];
};

} // namespace scene
} // namespace demo
