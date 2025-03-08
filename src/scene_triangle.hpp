// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once
#include "gl.hpp"

namespace demo {
namespace scene {

class Triangle {
public:
	Triangle() : mArray{0}, mBuffer{0} {}
	Triangle(const Triangle &) = delete;
	Triangle &operator=(const Triangle &) = delete;

	void Init();
	void Render(double time);

private:
	GLuint mArray;
	GLuint mBuffer;
};

} // namespace scene
} // namespace demo
