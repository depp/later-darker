cmake_minimum_required(VERSION 3.30)
project(later-darker)

set(out_name "LaterDarker")

# =============================================================================
# Common Settings
# =============================================================================

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED True)

set(gen ${CMAKE_CURRENT_BINARY_DIR}/src)
include_directories(${gen})

# =============================================================================
# Main Build
# =============================================================================

add_executable(Full WIN32
	"src/gl_debug.cpp"
	"src/gl_shader_data.cpp"
	"src/gl_shader_full.cpp"
	"src/gl_windows.cpp"
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
		${gen}/gl_api_full.cpp
		${gen}/gl_api_full.hpp
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

if(WIN32)
	target_link_libraries(Full PRIVATE opengl32.lib)
endif()

if(APPLE)
	target_link_libraries(Full PRIVATE "-framework OpenGL")
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
		${gen}/gl_api_compo.cpp
		${gen}/gl_api_compo.hpp
		${gen}/shader_data.cpp
	)
	target_compile_definitions(Compo PRIVATE COMPO)
	target_link_libraries(Compo PRIVATE glm::glm opengl32.lib)
	set_target_properties(Compo PROPERTIES OUTPUT_NAME "${out_name}Compo")
endif()

# =============================================================================
# Generated Source Code
# =============================================================================

set(DATA_TOOL ${CMAKE_CURRENT_SOURCE_DIR}/tools/target/release/tools${CMAKE_EXECUTABLE_SUFFIX}
	CACHE FILEPATH "Path to data generation tool.")

add_custom_command(
	OUTPUT
		src/shader_data.cpp
	COMMAND
		${DATA_TOOL}
		shader
		shader/shaders.txt
		${gen}/shader_data.cpp
	DEPENDS
		shader/cube.frag
		shader/cube.vert
		shader/shaders.txt
		shader/triangle.frag
		shader/triangle.vert
		${DATA_TOOL}
	WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_custom_command(
	OUTPUT
		src/gl_functions.txt
	COMMAND
		${DATA_TOOL}
		gl-scan
		--output=${gen}/gl_functions.txt
		${compo_sources}
	DEPENDS
		${compo_sources}
		${DATA_TOOL}
	WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_custom_command(
	OUTPUT
		src/gl_api_compo.hpp
		src/gl_api_compo.cpp
	COMMAND
		${DATA_TOOL}
		gl-emit
		--entry-points=${gen}/gl_functions.txt
		--output-header=${gen}/gl_api_compo.hpp
		--output-data=${gen}/gl_api_compo.cpp
	DEPENDS
		${gen}/gl_functions.txt
		${DATA_TOOL}
	WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_custom_command(
	OUTPUT
		src/gl_api_full.hpp
		src/gl_api_full.cpp
	COMMAND
		${DATA_TOOL}
		gl-emit
		--output-header=${gen}/gl_api_full.hpp
		--output-data=${gen}/gl_api_full.cpp
	DEPENDS
		${DATA_TOOL}
	WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_custom_target(sources ALL
	DEPENDS
	src/shader_data.cpp
	src/gl_functions.txt
	src/gl_api_compo.cpp
	src/gl_api_compo.hpp
	src/gl_api_full.cpp
	src/gl_api_full.hpp
)
