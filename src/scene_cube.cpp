// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "scene_cube.hpp"

#include "gl_shader.hpp"

#include <glm/gtc/matrix_transform.hpp>
#include <glm/gtc/quaternion.hpp>
#include <glm/gtc/type_ptr.hpp>
#include <glm/mat4x4.hpp>

#include <array>
#include <cmath>
#include <numbers>

namespace demo {
namespace scene {

namespace {

struct Vertex {
	short pos[4];
	unsigned char color[4];
};

const Vertex VertexData[6 * 4] = {
	// +x
	{{+1, -1, -1, 0}, {0x1d, 0x2b, 0x53, 0xff}},
	{{+1, +1, -1, 0}, {0x1d, 0x2b, 0x53, 0xff}},
	{{+1, -1, +1, 0}, {0x1d, 0x2b, 0x53, 0xff}},
	{{+1, +1, +1, 0}, {0x1d, 0x2b, 0x53, 0xff}},
	// -x
	{{-1, +1, -1, 0}, {0x7e, 0x25, 0x53, 0xff}},
	{{-1, -1, -1, 0}, {0x7e, 0x25, 0x53, 0xff}},
	{{-1, +1, +1, 0}, {0x7e, 0x25, 0x53, 0xff}},
	{{-1, -1, +1, 0}, {0x7e, 0x25, 0x53, 0xff}},
	// +y
	{{-1, +1, -1, 0}, {0x00, 0x75, 0x51, 0xff}},
	{{-1, +1, +1, 0}, {0x00, 0x75, 0x51, 0xff}},
	{{+1, +1, -1, 0}, {0x00, 0x75, 0x51, 0xff}},
	{{+1, +1, +1, 0}, {0x00, 0x75, 0x51, 0xff}},
	// -y
	{{-1, -1, +1, 0}, {0xff, 0x00, 0x4d, 0xff}},
	{{-1, -1, -1, 0}, {0xff, 0x00, 0x4d, 0xff}},
	{{+1, -1, +1, 0}, {0xff, 0x00, 0x4d, 0xff}},
	{{+1, -1, -1, 0}, {0xff, 0x00, 0x4d, 0xff}},
	// +z
	{{-1, -1, +1, 0}, {0xff, 0xa3, 0x00, 0xff}},
	{{+1, -1, +1, 0}, {0xff, 0xa3, 0x00, 0xff}},
	{{-1, +1, +1, 0}, {0xff, 0xa3, 0x00, 0xff}},
	{{+1, +1, +1, 0}, {0xff, 0xa3, 0x00, 0xff}},
	// -z
	{{+1, -1, -1, 0}, {0xff, 0xec, 0x27, 0xff}},
	{{-1, -1, -1, 0}, {0xff, 0xec, 0x27, 0xff}},
	{{+1, +1, -1, 0}, {0xff, 0xec, 0x27, 0xff}},
	{{-1, +1, -1, 0}, {0xff, 0xec, 0x27, 0xff}},
};

const unsigned short IndexData[6 * 4 + 5] = {
	0,  1,  2,  3,  0xffff, //
	4,  5,  6,  7,  0xffff, //
	8,  9,  10, 11, 0xffff, //
	12, 13, 14, 15, 0xffff, //
	16, 17, 18, 19, 0xffff, //
	20, 21, 22, 23          //
};

} // namespace

void Cube::Init() {
	glGenVertexArrays(1, &mArray);
	glBindVertexArray(mArray);
	glGenBuffers(2, mBuffer);
	glBindBuffer(GL_ARRAY_BUFFER, mBuffer[0]);
	glBufferData(GL_ARRAY_BUFFER, sizeof(VertexData), VertexData,
	             GL_STATIC_DRAW);
	glEnableVertexAttribArray(0);
	glVertexAttribPointer(0, 3, GL_SHORT, GL_FALSE, sizeof(Vertex),
	                      reinterpret_cast<void *>(0));
	glEnableVertexAttribArray(1);
	glVertexAttribPointer(1, 4, GL_UNSIGNED_BYTE, GL_TRUE, sizeof(Vertex),
	                      reinterpret_cast<void *>(8));
	glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, mBuffer[1]);
	glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(IndexData), IndexData,
	             GL_STATIC_DRAW);
}

void Cube::Render(double time) {
	glm::mat4 projection =
		glm::perspective(glm::radians(45.0f), 4.0f / 3.0f, 0.1f, 10.0f);
	const float fTime =
		static_cast<float>(std::fmod(time, 4.0 * std::numbers::pi));
	glm::mat4 modelView =
		glm::translate(
			glm::mat4(1.0f),
			glm::vec3(0.2f * std::cos(fTime), 0.2f * std::sin(fTime), -5.0f)) *
		glm::mat4_cast(
			glm::rotate(glm::rotate(glm::quat(1.0f, 0.0f, 0.0f, 0.0f), fTime,
	                                glm::vec3(0.0f, 1.0f, 0.0f)),
	                    0.5f * fTime, glm::vec3(0.0f, 0.0f, 1.0f)));
	glm::mat4 mvp = projection * modelView;

	glClearColor(0.2f, 0.2f, 0.2f, 1.0f);
	glClear(GL_COLOR_BUFFER_BIT);

	glUseProgram(gl_shader::CubeProgram);
	glUniformMatrix4fv(gl_shader::MVP, 1, GL_FALSE, glm::value_ptr(mvp));
	glPrimitiveRestartIndex(0xffff);
	glEnable(GL_PRIMITIVE_RESTART);
	glEnable(GL_CULL_FACE);
	glDrawElements(GL_TRIANGLE_STRIP, std::size(IndexData), GL_UNSIGNED_SHORT,
	               reinterpret_cast<void *>(0));
}

} // namespace scene
} // namespace demo
