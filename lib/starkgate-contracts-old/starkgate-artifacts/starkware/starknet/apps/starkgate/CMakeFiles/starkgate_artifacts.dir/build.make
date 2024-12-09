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

# Utility rule file for starkgate_artifacts.

# Include any custom commands dependencies for this target.
include src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_artifacts.dir/compiler_depend.make

# Include the progress variables for this target.
include src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_artifacts.dir/progress.make

src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_artifacts: src/starkware/starknet/apps/starkgate/artifacts/starkgate.stamp

src/starkware/starknet/apps/starkgate/artifacts/starkgate.stamp: src/starkware/starknet/apps/starkgate/copy_starkgate_artifacts
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --blue --bold --progress-dir=/app/build/Release/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Copying cairo and solidity compiled contracts to /app/build/Release/src/starkware/starknet/apps/starkgate/artifacts"
	cd /app/build/Release/src/starkware/starknet/apps/starkgate && rm -f /app/build/Release/src/starkware/starknet/apps/starkgate/artifacts/**.json
	cd /app/build/Release/src/starkware/starknet/apps/starkgate && ./copy_starkgate_artifacts --solidity_bridge_artifacts_dir /app/build/Release/src/starkware/starknet/apps/starkgate/eth/starkgate_bridge_sol_env/artifacts --cairo_bridge_artifacts_dir /app/build/Release/src/starkware/starknet/apps/starkgate/cairo/ --cairo_erc20_artifacts_dir /app/build/Release/src/starkware/starknet/std_contracts/ERC20/ --output_dir /app/build/Release/src/starkware/starknet/apps/starkgate/artifacts
	cd /app/build/Release/src/starkware/starknet/apps/starkgate && touch /app/build/Release/src/starkware/starknet/apps/starkgate/artifacts/starkgate.stamp

starkgate_artifacts: src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_artifacts
starkgate_artifacts: src/starkware/starknet/apps/starkgate/artifacts/starkgate.stamp
starkgate_artifacts: src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_artifacts.dir/build.make
.PHONY : starkgate_artifacts

# Rule to build all files generated by this target.
src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_artifacts.dir/build: starkgate_artifacts
.PHONY : src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_artifacts.dir/build

src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_artifacts.dir/clean:
	cd /app/build/Release/src/starkware/starknet/apps/starkgate && $(CMAKE_COMMAND) -P CMakeFiles/starkgate_artifacts.dir/cmake_clean.cmake
.PHONY : src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_artifacts.dir/clean

src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_artifacts.dir/depend:
	cd /app/build/Release && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /app /app/src/starkware/starknet/apps/starkgate /app/build/Release /app/build/Release/src/starkware/starknet/apps/starkgate /app/build/Release/src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_artifacts.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : src/starkware/starknet/apps/starkgate/CMakeFiles/starkgate_artifacts.dir/depend

