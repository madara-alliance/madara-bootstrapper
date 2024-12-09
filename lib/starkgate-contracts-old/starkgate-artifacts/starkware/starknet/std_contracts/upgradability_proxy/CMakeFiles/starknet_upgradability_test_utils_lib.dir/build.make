# CMAKE generated file: DO NOT EDIT!
# Generated by "Unix Makefiles" Generator, CMake Version 3.22

# Delete rule output on recipe failure.
.DELETE_ON_ERROR:

#=============================================================================
# Special targets provided by cmake.

# Disable implicit rules so canonical targets will work.
.SUFFIXES:

# Disable VCS-based implicit rules.
% : %,v

# Disable VCS-based implicit rules.
% : RCS/%

# Disable VCS-based implicit rules.
% : RCS/%,v

# Disable VCS-based implicit rules.
% : SCCS/s.%

# Disable VCS-based implicit rules.
% : s.%

.SUFFIXES: .hpux_make_needs_suffix_list

# Command-line flag to silence nested $(MAKE).
$(VERBOSE)MAKESILENT = -s

#Suppress display of executed commands.
$(VERBOSE).SILENT:

# A target that is always out of date.
cmake_force:
.PHONY : cmake_force

#=============================================================================
# Set environment variables for the build.

# The shell in which to execute make rules.
SHELL = /bin/sh

# The CMake executable.
CMAKE_COMMAND = /usr/local/lib/python3.7/site-packages/cmake/data/bin/cmake

# The command to remove a file.
RM = /usr/local/lib/python3.7/site-packages/cmake/data/bin/cmake -E rm -f

# Escaping for special characters.
EQUALS = =

# The top-level source directory on which CMake was run.
CMAKE_SOURCE_DIR = /app

# The top-level build directory on which CMake was run.
CMAKE_BINARY_DIR = /app/build/Release

# Utility rule file for starknet_upgradability_test_utils_lib.

# Include any custom commands dependencies for this target.
include src/starkware/starknet/std_contracts/upgradability_proxy/CMakeFiles/starknet_upgradability_test_utils_lib.dir/compiler_depend.make

# Include the progress variables for this target.
include src/starkware/starknet/std_contracts/upgradability_proxy/CMakeFiles/starknet_upgradability_test_utils_lib.dir/progress.make

src/starkware/starknet/std_contracts/upgradability_proxy/CMakeFiles/starknet_upgradability_test_utils_lib: python_libs/starknet_upgradability_test_utils_lib.info

python_libs/starknet_upgradability_test_utils_lib.info: ../../src/cmake_utils/gen_py_lib.py
python_libs/starknet_upgradability_test_utils_lib.info: python_libs/pip_cairo_lang.info
python_libs/starknet_upgradability_test_utils_lib.info: src/starkware/starknet/std_contracts/upgradability_proxy/starknet_upgradability_test_utils_lib_copy_files.stamp
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --blue --bold --progress-dir=/app/build/Release/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Generating ../../../../../python_libs/starknet_upgradability_test_utils_lib.info"
	cd /app/build/Release/src/starkware/starknet/std_contracts/upgradability_proxy && ../../../../../../../src/cmake_utils/gen_py_lib.py --name starknet_upgradability_test_utils_lib --lib_dir /app/build/Release/src/starkware/starknet/std_contracts/upgradability_proxy/starknet_upgradability_test_utils_lib --files starkware/starknet/std_contracts/upgradability_proxy/test_utils.py --lib_deps pip_cairo_lang --py_exe_deps --cmake_dir src/starkware/starknet/std_contracts/upgradability_proxy --prefix starkware/starknet/std_contracts/upgradability_proxy/ --output /app/build/Release/python_libs/starknet_upgradability_test_utils_lib.info

starknet_upgradability_test_utils_lib: python_libs/starknet_upgradability_test_utils_lib.info
starknet_upgradability_test_utils_lib: src/starkware/starknet/std_contracts/upgradability_proxy/CMakeFiles/starknet_upgradability_test_utils_lib
starknet_upgradability_test_utils_lib: src/starkware/starknet/std_contracts/upgradability_proxy/CMakeFiles/starknet_upgradability_test_utils_lib.dir/build.make
.PHONY : starknet_upgradability_test_utils_lib

# Rule to build all files generated by this target.
src/starkware/starknet/std_contracts/upgradability_proxy/CMakeFiles/starknet_upgradability_test_utils_lib.dir/build: starknet_upgradability_test_utils_lib
.PHONY : src/starkware/starknet/std_contracts/upgradability_proxy/CMakeFiles/starknet_upgradability_test_utils_lib.dir/build

src/starkware/starknet/std_contracts/upgradability_proxy/CMakeFiles/starknet_upgradability_test_utils_lib.dir/clean:
	cd /app/build/Release/src/starkware/starknet/std_contracts/upgradability_proxy && $(CMAKE_COMMAND) -P CMakeFiles/starknet_upgradability_test_utils_lib.dir/cmake_clean.cmake
.PHONY : src/starkware/starknet/std_contracts/upgradability_proxy/CMakeFiles/starknet_upgradability_test_utils_lib.dir/clean

src/starkware/starknet/std_contracts/upgradability_proxy/CMakeFiles/starknet_upgradability_test_utils_lib.dir/depend:
	cd /app/build/Release && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /app /app/src/starkware/starknet/std_contracts/upgradability_proxy /app/build/Release /app/build/Release/src/starkware/starknet/std_contracts/upgradability_proxy /app/build/Release/src/starkware/starknet/std_contracts/upgradability_proxy/CMakeFiles/starknet_upgradability_test_utils_lib.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : src/starkware/starknet/std_contracts/upgradability_proxy/CMakeFiles/starknet_upgradability_test_utils_lib.dir/depend

