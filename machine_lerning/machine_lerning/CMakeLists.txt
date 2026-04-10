# CMakeList.txt : CMake project for machine_lerning, include source and define
# project specific logic here.
#

# Add source to this project's executable.
add_executable (machine_lerning "machine_lerning.cpp" "machine_lerning.h" "lodepng.cpp")

if (CMAKE_VERSION VERSION_GREATER 3.12)
  set_property(TARGET machine_lerning PROPERTY CXX_STANDARD 20)
endif()

# TODO: Add tests and install targets if needed.
