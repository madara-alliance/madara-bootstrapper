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

# Utility rule file for solidity_contracts_components_sol_dryrun.

# Include any custom commands dependencies for this target.
include src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_dryrun.dir/compiler_depend.make

# Include the progress variables for this target.
include src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_dryrun.dir/progress.make

src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_dryrun: python_libs/solidity_contracts_components_sol.info.dryrun

python_libs/solidity_contracts_components_sol.info.dryrun: ../../src/cmake_utils/gen_py_lib.py
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --blue --bold --progress-dir=/app/build/Release/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Generating ../../../../python_libs/solidity_contracts_components_sol.info.dryrun"
	cd /app/build/Release/src/starkware/solidity/components && ../../../../../../src/cmake_utils/gen_py_lib.py --name solidity_contracts_components_sol --lib_dir /app/build/Release/src/starkware/solidity/components/solidity_contracts_components_sol --files starkware/solidity/components/FactRegistry.sol starkware/solidity/components/GovernedFinalizable.sol --lib_deps solidity_contracts_interfaces_sol --py_exe_deps --cmake_dir src/starkware/solidity/components --prefix starkware/solidity/components/ --output /app/build/Release/python_libs/solidity_contracts_components_sol.info.dryrun

solidity_contracts_components_sol_dryrun: python_libs/solidity_contracts_components_sol.info.dryrun
solidity_contracts_components_sol_dryrun: src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_dryrun
solidity_contracts_components_sol_dryrun: src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_dryrun.dir/build.make
.PHONY : solidity_contracts_components_sol_dryrun

# Rule to build all files generated by this target.
src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_dryrun.dir/build: solidity_contracts_components_sol_dryrun
.PHONY : src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_dryrun.dir/build

src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_dryrun.dir/clean:
	cd /app/build/Release/src/starkware/solidity/components && $(CMAKE_COMMAND) -P CMakeFiles/solidity_contracts_components_sol_dryrun.dir/cmake_clean.cmake
.PHONY : src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_dryrun.dir/clean

src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_dryrun.dir/depend:
	cd /app/build/Release && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /app /app/src/starkware/solidity/components /app/build/Release /app/build/Release/src/starkware/solidity/components /app/build/Release/src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_dryrun.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_dryrun.dir/depend

