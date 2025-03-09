// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "gl.hpp"
#include "gl_shader.hpp"
#include "log.hpp"
#include "os_string.hpp"
#include "scene_triangle.hpp"
#include "var.hpp"

#include <GLFW/glfw3.h>

#include <cstdio>

#define UNICODE 1
#define WIN32_LEAN_AND_MEAN 1
#include <Windows.h>
#include <shellapi.h>

namespace demo {
namespace {

extern "C" void ErrorCallback(int error, const char *description) {
	(void)error;
	std::string message;
	message.append("GLFW error: ");
	message.append(description);
	std::wstring wmessage = ToOSString(message);
	MessageBoxW(nullptr, wmessage.c_str(), nullptr, MB_ICONSTOP);
}

void Main() {
	log::Init();
	log::Log(log::Level::Info, "Opened console");
	log::Log(log::Level::Debug,
	         "A debug message; Unicode: \xce\xb1\xce\xb2"); // alpha, beta
	log::Log(log::Level::Warn, "A warning");
	log::Log(log::Level::Error, "An error message");

	glfwSetErrorCallback(ErrorCallback);
	if (!glfwInit()) {
		std::exit(1);
	}

	// All of these are necessary.
	//
	// - On Apple devices, context will be version 2.1 if no hints are provided.
	//   FORWARD_COMPAT, PROFILE, and VERSION are all required to get a
	//   different result. The result is the highest version, probably
	//   either 3.3 or 4.1.
	//
	// - On Mesa, 3.0 is the maximum without FORWARD_COMPAT, and 3.1 is the
	//   maximum with FORWARD_COMPAT but without CORE_PROFILE.
	//
	// - With AMD or Nvidia drivers on Linux or Windows, you will always get the
	//   highest version supported even without any hints.
	glfwWindowHint(GLFW_OPENGL_FORWARD_COMPAT, GLFW_TRUE);
	glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
	glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
	glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);

	if (var::DebugContext) {
		glfwWindowHint(GLFW_OPENGL_DEBUG_CONTEXT, GLFW_TRUE);
	}

	GLFWwindow *window =
		glfwCreateWindow(640, 480, "Later, Darker", nullptr, nullptr);
	if (window == nullptr) {
		glfwTerminate();
		exit(1);
	}

	glfwMakeContextCurrent(window);
	gladLoadGL(glfwGetProcAddress); // TODO: Log version.
	gl_shader::Init();
	scene::Triangle scene;
	scene.Init();

	glfwSwapInterval(1);

	while (!glfwWindowShouldClose(window)) {
		int width, height;
		glfwGetFramebufferSize(window, &width, &height);
		glViewport(0, 0, width, height);

		double time = glfwGetTime();
		scene.Render(time);

		glfwSwapBuffers(window);
		glfwPollEvents();
	}

	glfwDestroyWindow(window);

	glfwTerminate();
}

} // namespace

// ============================================================================
// Windows
// ============================================================================

namespace {

void ParseCommandLine(const wchar_t *cmdLine) {
	int nArgs;
	wchar_t **args = CommandLineToArgvW(cmdLine, &nArgs);
	if (args == nullptr) {
		std::abort();
	}
	ParseCommandArguments(nArgs, args);
	LocalFree(args);
}

} // namespace

} // namespace demo

int WINAPI wWinMain(_In_ HINSTANCE hInstance, _In_opt_ HINSTANCE hPrevInstance,
                    _In_ LPWSTR lpCmdLine, _In_ int nShowCmd) {
	(void)hInstance;
	(void)hPrevInstance;
	(void)nShowCmd;
	demo::ParseCommandLine(lpCmdLine);
	demo::Main();
	return 0;
}
