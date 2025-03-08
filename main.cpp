#include <GLFW/glfw3.h>

#include <cstdio>
#include <cstdlib>

namespace
{

extern "C" void ErrorCallback(int error, const char *description)
{
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

	while (!glfwWindowShouldClose(window)) {
		glfwSwapBuffers(window);
		glfwPollEvents();
	}

	glfwTerminate();
}
