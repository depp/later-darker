// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "main.hpp"

#include "gl.hpp"
#include "gl_shader.hpp"
#include "log.hpp"
#include "scene_cube.hpp"

#include <cmath>

#define NOMINMAX 1
#undef UNICODE
#define WIN32_LEAN_AND_MEAN 1
#include <Windows.h>

namespace demo {

namespace gl_api {

[[noreturn]]
void MissingFunction(const char *name) {
	(void)name;
	FAIL("Missing OpenGL entry point.", log::Attr{"name", name});
}

// Load OpenGL entry points.
void LoadProcs() {
	const char *namePtr = FunctionNames;
	for (int i = 0; i < FunctionPointerCount; i++) {
		PROC proc = wglGetProcAddress(namePtr);
		CHECK(proc != nullptr);
		FunctionPointers[i] = static_cast<void *>(proc);
		namePtr += std::strlen(namePtr) + 1;
	}
}

} // namespace gl_api

namespace {

constexpr const char *ClassName = "Demo";
constexpr const char *WindowTitle = "Later, Darker";
constexpr bool Fullscreen = true;
HWND Window;
HDC DeviceContext;

LRESULT CALLBACK WindowProc(HWND hWnd, UINT uMsg, WPARAM wParam,
                            LPARAM lParam) {
	switch (uMsg) {
	case WM_DESTROY:
		PostQuitMessage(0);
		break;

	case WM_SETCURSOR:
		// Hide cursor.
		if (LOWORD(lParam) == HTCLIENT) {
			SetCursor(nullptr);
		}
		break;

	case WM_KEYDOWN:
		if (wParam == VK_ESCAPE) {
			PostQuitMessage(0);
		}
		break;

	default:
		return DefWindowProcA(hWnd, uMsg, wParam, lParam);
	}

	return 0;
}

const PIXELFORMATDESCRIPTOR PixelFormatDescriptor = {
	.nSize = sizeof(PIXELFORMATDESCRIPTOR),
	.nVersion = 1,
	.dwFlags = PFD_DOUBLEBUFFER | PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL,
	.iPixelType = PFD_TYPE_RGBA,
	.cColorBits = 24,
	.iLayerType = PFD_MAIN_PLANE,
};

void InitWGL() {
	const HDC dc = GetDC(Window);
	DeviceContext = dc;
	const int pixelFormat = ChoosePixelFormat(dc, &PixelFormatDescriptor);
	if (pixelFormat == 0) {
		FAIL("Could not choose pixel format.");
	}
	if (!SetPixelFormat(dc, pixelFormat, &PixelFormatDescriptor)) {
		FAIL("Could not set pixel format.");
	}
	const HGLRC rc = wglCreateContext(dc);
	if (rc == nullptr) {
		FAIL("Failed to create context.");
	}
	if (!wglMakeCurrent(dc, rc)) {
		FAIL("Faled to make context current.");
	}
	gl_api::LoadProcs();
}

void CreateMainWindow(int nShowCmd) {
	HINSTANCE hInstance = GetModuleHandleA(nullptr);

	WNDCLASSA wc = {
		.lpfnWndProc = WindowProc,
		.hInstance = hInstance,
		.lpszClassName = ClassName,
	};
	if (!RegisterClassA(&wc)) {
		FAIL("Failed to register window class.");
	}

	if constexpr (Fullscreen) {
		// Alternatively, we could MonitorFromPoint() with (0,0), which gives us
		// the primary monitor, then GetMonitorInfo().
		int width = GetSystemMetrics(SM_CXSCREEN);
		int height = GetSystemMetrics(SM_CYSCREEN);

		// Borderless fullscreen window style.
		constexpr DWORD style = WS_POPUP | WS_VISIBLE;
		Window = CreateWindowA(ClassName, WindowTitle, style, 0, 0, width,
		                       height, nullptr, nullptr, hInstance, nullptr);
		if (Window == nullptr) {
			FAIL("Failed to create window.");
		}
	} else {
		constexpr DWORD style = WS_OVERLAPPEDWINDOW;
		Window = CreateWindowA(ClassName, WindowTitle, style, CW_USEDEFAULT,
		                       CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT,
		                       nullptr, nullptr, hInstance, nullptr);
		if (Window == nullptr) {
			FAIL("Failed to create window.");
		}
	}

	ShowWindow(Window, nShowCmd);
	// Are SetForegroundWindow and SetFocus necessary?
	SetForegroundWindow(Window);
	SetFocus(Window);
	InitWGL();
}

void Main() {
	gl_shader::Init();
	scene::Cube scene;
	scene.Init();
	const unsigned long long baseTime = GetTickCount64();
	for (;;) {
		MSG msg;
		if (PeekMessage(&msg, nullptr, 0, 0, PM_REMOVE)) {
			if (msg.message == WM_QUIT) {
				break;
			}
			TranslateMessage(&msg);
			DispatchMessageA(&msg);
		} else {
			const unsigned long long currentTicks = GetTickCount64() - baseTime;
			const double time =
				static_cast<double>(static_cast<int>(currentTicks)) * 0.001;
			scene.Render(time);
			SwapBuffers(DeviceContext);
			Sleep(5);
		}
	}
}

} // namespace
} // namespace demo

int WINAPI WinMain(_In_ HINSTANCE hInstance, _In_opt_ HINSTANCE hPrevInstance,
                   _In_ LPSTR lpCmdLine, _In_ int nShowCmd) {
	(void)hInstance;
	(void)hPrevInstance;
	(void)lpCmdLine;
	(void)nShowCmd;
	demo::CreateMainWindow(nShowCmd);
	demo::Main();
	return 0;
}
