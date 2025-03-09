cmake_minimum_required(VERSION 3.30)
project(later-darker)

include(FetchContent)
FetchContent_Declare(
	glad
	URL https://github.com/Dav1dde/glad/archive/refs/tags/v2.0.8.tar.gz
	URL_HASH SHA256=44f06f9195427c7017f5028d0894f57eb216b0a8f7c4eda7ce883732aeb2d0fc
)
FetchContent_MakeAvailable(glad)

add_subdirectory("${glad_SOURCE_DIR}/cmake" glad_cmake)
glad_add_library(glad_gl33 SHARED REPRODUCIBLE API gl:core=3.3)

find_package(glfw3 CONFIG REQUIRED)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED True)

add_executable(later-darker WIN32
	"src/gl_shader.cpp"
	"src/log.cpp"
	"src/main.cpp"
	"src/os_string.cpp"
	"src/scene_triangle.cpp"
	"src/text_buffer.cpp"
	"src/var.cpp"
	"src/wide_text_buffer.cpp"
)

if(MSVC)
	target_compile_options(later-darker PRIVATE /W4)
else()
	target_compile_options(later-darker PRIVATE -Wall -Wextra)
endif()

target_link_libraries(later-darker PRIVATE glfw glad_gl33)
