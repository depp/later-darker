cmake_minimum_required(VERSION 3.30)
project(later-darker)

set(out_name "LaterDarker")

# =============================================================================
# Dependencies
# =============================================================================

set(USE_GLAD 1)
if(APPLE)
	set(USE_GLAD 0)
endif()

# This is for GLAD, and it allows GLAD to use a virtualenv Python if found
# first, which means that GLAD can find jinja2 in such an environment.
if(NOT DEFINED Python_FIND_FRAMEWORK)
	set(Python_FIND_FRAMEWORK NEVER)
endif()

include(FetchContent)
FetchContent_Declare(
	glad
	URL https://github.com/Dav1dde/glad/archive/refs/tags/v2.0.8.tar.gz
	URL_HASH SHA256=44f06f9195427c7017f5028d0894f57eb216b0a8f7c4eda7ce883732aeb2d0fc
)

set(FETCH_PKGS)
if(USE_GLAD)
	list(APPEND FETCH_PKGS glad)
endif()

FetchContent_MakeAvailable(${FETCH_PKGS})

# =============================================================================
# Common Settings
# =============================================================================

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED True)

set(gen ${CMAKE_CURRENT_BINARY_DIR}/src)

# =============================================================================
# Main Build
# =============================================================================

add_executable(Full WIN32
	"src/gl_debug.cpp"
	"src/gl_shader_data.cpp"
	"src/gl_shader_dev.cpp"
	"src/log_standard.cpp"
	"src/main.cpp"
	"src/os_string.cpp"
	"src/scene_cube.cpp"
	"src/scene_triangle.cpp"
	"src/text_buffer.cpp"
	"src/text_unicode.cpp"
	"src/var.cpp"
	${gen}/shader_data.cpp
)

if(WIN32)
	target_sources(Full PRIVATE
		"src/log_windows.cpp"
		"src/os_file_windows.cpp"
		"src/os_windows.cpp"
		"src/wide_text_buffer.cpp"
	)
else()
	target_sources(Full PRIVATE
		"src/log_unix.cpp"
		"src/os_file_unix.cpp"
		"src/os_unix.cpp"
	)
endif()

if(MSVC)
	target_compile_options(Full PRIVATE "/utf-8")
	target_compile_options(Full PRIVATE /W4)
else()
	target_compile_options(Full PRIVATE -Wall -Wextra)
endif()

find_package(glfw3 CONFIG REQUIRED)
target_link_libraries(Full PRIVATE glfw)

find_package(glm CONFIG REQUIRED)
target_link_libraries(Full PRIVATE glm::glm)

if(APPLE)
	target_link_libraries(Full PRIVATE "-framework OpenGL")
endif()

if(USE_GLAD)
	add_subdirectory("${glad_SOURCE_DIR}/cmake" glad_cmake)
	glad_add_library(glad_gl33 SHARED REPRODUCIBLE API gl:core=3.3 EXTENSIONS GL_KHR_debug)
	target_link_libraries(Full PRIVATE glad_gl33)
endif()

set_target_properties(Full PROPERTIES OUTPUT_NAME "${out_name}Full")

# =============================================================================
# Competition Build
# =============================================================================

set(compo_sources
	src/gl_shader_compo.cpp
	src/gl_shader_data.cpp
	src/main_windows_compo.cpp
	src/scene_cube.cpp
	src/scene_triangle.cpp
)

if(WIN32)
	add_executable(Compo WIN32
		${compo_sources}
		${gen}/shader_data.cpp
	)
	target_compile_definitions(Compo PRIVATE COMPO)
	target_link_libraries(Compo PRIVATE glm::glm glad_gl33 opengl32.lib)
	set_target_properties(Compo PROPERTIES OUTPUT_NAME "${out_name}Compo")
endif()

# =============================================================================
# Generated Source Code
# =============================================================================

add_custom_command(
	OUTPUT
		src/shader_data.cpp
	COMMAND
		${CMAKE_CURRENT_SOURCE_DIR}/tools/target/release/tools${CMAKE_EXECUTABLE_SUFFIX}
		shader
		${CMAKE_CURRENT_SOURCE_DIR}/shader/shaders.txt
		${gen}/shader_data.cpp
	DEPENDS
		shader/cube.frag
		shader/cube.vert
		shader/shaders.txt
		shader/triangle.frag
		shader/triangle.vert
)
