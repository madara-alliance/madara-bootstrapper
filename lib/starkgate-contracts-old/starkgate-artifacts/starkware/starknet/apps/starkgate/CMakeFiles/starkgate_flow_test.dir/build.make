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

# Utility rule file for starkgate_flow_test.

# Include any custom commands dependencies for this target.
include src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_flow_test.dir/compiler_depend.make

# Include the progress variables for this target.
include src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_flow_test.dir/progress.make

src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_flow_test: python_libs/starkgate_flow_test.info

python_libs/starkgate_flow_test.info: ../../src/cmake_utils/gen_python_exe.py
python_libs/starkgate_flow_test.info: python_libs/starkgate_flow_test_venv.info
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --blue --bold --progress-dir=/app/build/Release/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Generating ../../../../../python_libs/starkgate_flow_test.info"
	cd /app/build/Release/src/starkware/starknet/apps/starkgate && ../../../../../../../src/cmake_utils/gen_python_exe.py --name starkgate_flow_test --exe_path /app/build/Release/src/starkware/starknet/apps/starkgate/starkgate_flow_test --venv starkgate_flow_test_venv --module pytest --args=\ {VENV_SITE_DIR}/starkware/starknet/apps/starkgate\  --info_dir /app/build/Release/python_libs --cmake_binary_dir /app/build/Release --working_dir=/app/build/Release --environment_variables=

starkgate_flow_test: python_libs/starkgate_flow_test.info
starkgate_flow_test: src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_flow_test
starkgate_flow_test: src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_flow_test.dir/build.make
.PHONY : starkgate_flow_test

# Rule to build all files generated by this target.
src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_flow_test.dir/build: starkgate_flow_test
.PHONY : src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_flow_test.dir/build

src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_flow_test.dir/clean:
	cd /app/build/Release/src/starkware/starknet/apps/starkgate && $(CMAKE_COMMAND) -P CMakeFiles/starkgate_flow_test.dir/cmake_clean.cmake
.PHONY : src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_flow_test.dir/clean

src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_flow_test.dir/depend:
	cd /app/build/Release && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /app /app/src/starkware/starknet/apps/starkgate /app/build/Release /app/build/Release/src/starkware/starknet/apps/starkgate /app/build/Release/src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_flow_test.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_flow_test.dir/depend

