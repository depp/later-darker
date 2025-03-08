cmake_minimum_required(VERSION 3.30)
project(later-darker)

find_package(OpenGL REQUIRED)
find_package(glfw3 CONFIG REQUIRED)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED True)

add_executable(later-darker main.cpp)

if(MSVC)
	target_compile_options(later-darker PRIVATE /W4)
else()
	target_compile_options(later-darker PRIVATE -Wall -Wextra)
endif()

target_link_libraries(later-darker PRIVATE glfw OpenGL::GL)
