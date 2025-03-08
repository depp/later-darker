// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once
#include "gl.hpp"

namespace demo
{

class TriangleScene
{
public:
	TriangleScene() : mArray{0}, mBuffer{0} {}
	TriangleScene(const TriangleScene &) = delete;
	TriangleScene &operator=(const TriangleScene &) = delete;

	void Init();
	void Render(double time);

private:
	GLuint mArray;
	GLuint mBuffer;
};

} // namespace demo
