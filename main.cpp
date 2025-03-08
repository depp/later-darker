#include <GLFW/glfw3.h>
#include <gl/GL.h>

#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <numbers>

namespace
{

extern "C" void ErrorCallback(int error, const char *description)
{
	(void)error;
	std::fprintf(stderr, "Error: %s\n", description);
}

} // namespace

int main(int argc, char **argv)
{
	(void)argc;
	(void)argv;

	glfwSetErrorCallback(ErrorCallback);
	if (!glfwInit()) {
		std::exit(1);
	}

	GLFWwindow *window =
		glfwCreateWindow(640, 480, "Later, Darker", nullptr, nullptr);
	if (window == nullptr) {
		glfwTerminate();
		exit(1);
	}

	glfwMakeContextCurrent(window);
	glfwSwapInterval(1);

	while (!glfwWindowShouldClose(window)) {
		int width, height;
		glfwGetFramebufferSize(window, &width, &height);
		glViewport(0, 0, width, height);

		double time = glfwGetTime();
		constexpr float d = std::numbers::pi_v<float> * 2.0f / 3.0f;
		constexpr double rate = 0.3;
		float a = static_cast<float>(std::fmod(time * rate, 1.0)) *
		          (2.0f * std::numbers::pi_v<float>);
		glClearColor(0.5f + 0.5f * std::sin(a + d), 0.5f + 0.5f * std::sin(a),
		             0.5f + 0.5f * std::sin(a - d), 1.0f);
		glClear(GL_COLOR_BUFFER_BIT);

		glfwSwapBuffers(window);
		glfwPollEvents();
	}

	glfwDestroyWindow(window);

	glfwTerminate();
}
