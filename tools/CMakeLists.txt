cmake_minimum_required(VERSION 3.30)
project(later-darker-tools)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED True)

add_executable(shader
	"shader.cpp"
)
