// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "main.hpp"

#include "log.hpp"

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

LRESULT CALLBACK WindowProc(HWND hWnd, UINT uMsg, WPARAM wParam,
                            LPARAM lParam) {
	switch (uMsg) {
	case WM_DESTROY:
		PostQuitMessage(0);
		return 0;
	case WM_PAINT: {
		PAINTSTRUCT ps;
		HDC dc = BeginPaint(Window, &ps);
		FillRect(dc, &ps.rcPaint, (HBRUSH)(COLOR_WINDOW + 1));
		EndPaint(Window, &ps);
	}
		return 0;
	}
	return DefWindowProcA(hWnd, uMsg, wParam, lParam);
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
		// https://devblogs.microsoft.com/oldnewthing/20070809-00/?p=25643
		// The primary monitor contains point (0, 0).
		HMONITOR monitor =
			MonitorFromPoint(POINT{0, 0}, MONITOR_DEFAULTTOPRIMARY);
		if (monitor == nullptr) {
			FAIL("Failed to get main monitor.");
		}
		MONITORINFO mi;
		mi.cbSize = sizeof(mi);
		if (!GetMonitorInfo(monitor, &mi)) {
			FAIL("Failed to get monitor information.");
		}

		// Borderless fullscreen window style.
		constexpr DWORD style = WS_POPUP | WS_VISIBLE;
		Window = CreateWindowA(ClassName, WindowTitle, style, mi.rcMonitor.left,
		                       mi.rcMonitor.top,
		                       mi.rcMonitor.right - mi.rcMonitor.left,
		                       mi.rcMonitor.bottom - mi.rcMonitor.top, nullptr,
		                       nullptr, hInstance, nullptr);
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
}

void Main() {
	MSG msg;
	while (GetMessageA(&msg, nullptr, 0, 0)) {
		TranslateMessage(&msg);
		DispatchMessageA(&msg);
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
