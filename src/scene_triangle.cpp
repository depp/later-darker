// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "scene_triangle.hpp"

#include "gl_shader.hpp"

#include <cmath>
#include <numbers>

namespace demo {
namespace scene {

namespace {

constexpr float InverseAspect = 480.0f / 640.0f;
constexpr float TriangleSize = 0.8f;

const float VertexData[6] = {
	0.0f,
	TriangleSize,
	-std::sqrt(3.0f) * 0.5f * TriangleSize *InverseAspect,
	-0.5f,
	std::sqrt(3.0f) * 0.5f * TriangleSize *InverseAspect,
	-0.5f,
};

} // namespace

void Triangle::Init() {
	glGenVertexArrays(1, &mArray);
	glBindVertexArray(mArray);
	glGenBuffers(1, &mBuffer);
	glBindBuffer(GL_ARRAY_BUFFER, mBuffer);
	glBufferData(GL_ARRAY_BUFFER, sizeof(VertexData), VertexData,
	             GL_STATIC_DRAW);
	glEnableVertexAttribArray(0);
	glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, 8, 0);
}

void Triangle::Render(double time) {
	constexpr float d = std::numbers::pi_v<float> * 2.0f / 3.0f;
	constexpr double rate = 0.3;
	float a = static_cast<float>(std::fmod(time * rate, 1.0)) *
	          (2.0f * std::numbers::pi_v<float>);
	glClearColor(0.5f + 0.5f * std::sin(a + d), 0.5f + 0.5f * std::sin(a),
	             0.5f + 0.5f * std::sin(a - d), 1.0f);
	glClear(GL_COLOR_BUFFER_BIT);

	glUseProgram(demo::gl_shader::TriangleProgram);
	glDrawArrays(GL_TRIANGLES, 0, 3);
}

} // namespace scene
} // namespace demo
