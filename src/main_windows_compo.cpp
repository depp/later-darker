// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "main.hpp"

#include "gl.hpp"
#include "log.hpp"

#include <cmath>

#define NOMINMAX 1
#undef UNICODE
#define WIN32_LEAN_AND_MEAN 1
#include <Windows.h>

namespace demo {
namespace {

constexpr const char *ClassName = "Demo";
constexpr const char *WindowTitle = "Later, Darker";
constexpr bool Fullscreen = true;
HWND Window;
HMODULE OpenGL;
HDC DeviceContext;

LRESULT CALLBACK WindowProc(HWND hWnd, UINT uMsg, WPARAM wParam,
                            LPARAM lParam) {
	switch (uMsg) {
	case WM_DESTROY:
		PostQuitMessage(0);
		return 0;

	case WM_SETCURSOR:
		// Hide cursor.
		if (LOWORD(lParam) == HTCLIENT) {
			SetCursor(nullptr);
		}
	}
	return DefWindowProcA(hWnd, uMsg, wParam, lParam);
}

const PIXELFORMATDESCRIPTOR PixelFormatDescriptor = {
	.nSize = sizeof(PIXELFORMATDESCRIPTOR),
	.nVersion = 1,
	.dwFlags = PFD_DOUBLEBUFFER | PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL,
	.iPixelType = PFD_TYPE_RGBA,
	.cColorBits = 24,
	.iLayerType = PFD_MAIN_PLANE,
};

GLADapiproc GetGLProcAddress(const char *name) {
	PROC proc = wglGetProcAddress(name);
	if (proc == nullptr) {
		proc = GetProcAddress(OpenGL, name);
	}
	return reinterpret_cast<GLADapiproc>(proc);
}

void InitWGL() {
	OpenGL = GetModuleHandleA("opengl32.dll");
	if (OpenGL == nullptr) {
		FAIL("Could not get OpenGL DLL handle.");
	}
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
	gladLoadGL(GetGLProcAddress);
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
	InitWGL();
}

void Main() {
	double time = 0.0;
	for (;;) {
		MSG msg;
		if (PeekMessage(&msg, nullptr, 0, 0, PM_REMOVE)) {
			if (msg.message == WM_QUIT) {
				break;
			}
			TranslateMessage(&msg);
			DispatchMessageA(&msg);
		} else {
			time += 0.01;
			const float a = 0.5f + 0.5f * static_cast<float>(std::sin(time));
			glClearColor(a, a, a, 1.0f);
			glClear(GL_COLOR_BUFFER_BIT);
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
