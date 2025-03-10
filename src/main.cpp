// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "main.hpp"

#include "gl.hpp"
#include "gl_shader.hpp"
#include "log.hpp"
#include "os_string.hpp"
#include "scene_triangle.hpp"
#include "var.hpp"

#include <GLFW/glfw3.h>

#include <array>
#include <cassert>
#include <cstdio>

#define UNICODE 1
#define WIN32_LEAN_AND_MEAN 1
#include <Windows.h>
#include <shellapi.h>

#define FAIL_GLFW(...) FAIL(__VA_ARGS__, GLFWErrorInfo::Get())

namespace demo {
namespace {

// Information about GLFW errors to add to log messages.
class GLFWErrorInfo {
public:
	static GLFWErrorInfo Get() {
		const char *description;
		int error = glfwGetError(&description);
		if (error == 0) {
			return GLFWErrorInfo{};
		}
		return GLFWErrorInfo{error, description};
	}

	GLFWErrorInfo() : mError{0}, mDescription{} {}
	GLFWErrorInfo(int error, const char *description)
		: mError{error}, mDescription{description} {}

	void AddToRecord(log::Record &record) const {
		record.Add("domain", "GLFW");
		if (mError != 0) {
			record.Add("error", mError);
			record.Add("description", mDescription);
		}
	}

private:
	int mError;
	std::string_view mDescription;
};

extern "C" void ErrorCallback(int error, const char *description) {
	log::Record{log::Level::Error, log::Location::Zero, "GLFW error.",
	            GLFWErrorInfo{error, description}}
		.Log();
}

void Main() {
	log::Init();

	glfwSetErrorCallback(ErrorCallback);
	if (!glfwInit()) {
		FAIL_GLFW("Could not initialize GLFW.");
	}

	// All of these are necessary.
	//
	// - On Apple devices, context will be version 2.1 if no hints are
	// provided.
	//   FORWARD_COMPAT, PROFILE, and VERSION are all required to get a
	//   different result. The result is the highest version, probably
	//   either 3.3 or 4.1.
	//
	// - On Mesa, 3.0 is the maximum without FORWARD_COMPAT, and 3.1 is the
	//   maximum with FORWARD_COMPAT but without CORE_PROFILE.
	//
	// - With AMD or Nvidia drivers on Linux or Windows, you will always get
	// the
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
		FAIL_GLFW("Could not create window.");
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

[[noreturn]]
void ExitError() {
	glfwTerminate();
	ExitProcess(1);
}

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
