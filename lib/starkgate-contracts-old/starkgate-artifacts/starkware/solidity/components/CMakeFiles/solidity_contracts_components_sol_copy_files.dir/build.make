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

# Utility rule file for solidity_contracts_components_sol_copy_files.

# Include any custom commands dependencies for this target.
include src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_copy_files.dir/compiler_depend.make

# Include the progress variables for this target.
include src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_copy_files.dir/progress.make

src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_copy_files: src/starkware/solidity/components/solidity_contracts_components_sol_copy_files.stamp

src/starkware/solidity/components/solidity_contracts_components_sol_copy_files.stamp: src/starkware/solidity/components/solidity_contracts_components_sol/starkware/solidity/components/FactRegistry.sol
src/starkware/solidity/components/solidity_contracts_components_sol_copy_files.stamp: src/starkware/solidity/components/solidity_contracts_components_sol/starkware/solidity/components/GovernedFinalizable.sol
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --blue --bold --progress-dir=/app/build/Release/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Generating solidity_contracts_components_sol_copy_files.stamp"
	cd /app/build/Release/src/starkware/solidity/components && /usr/local/lib/python3.7/site-packages/cmake/data/bin/cmake -E touch /app/build/Release/src/starkware/solidity/components/solidity_contracts_components_sol_copy_files.stamp

src/starkware/solidity/components/solidity_contracts_components_sol/starkware/solidity/components/FactRegistry.sol: ../../src/starkware/solidity/components/FactRegistry.sol
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --blue --bold --progress-dir=/app/build/Release/CMakeFiles --progress-num=$(CMAKE_PROGRESS_2) "Copying file FactRegistry.sol"
	cd /app/build/Release/src/starkware/solidity/components && /usr/local/lib/python3.7/site-packages/cmake/data/bin/cmake -E copy /app/src/starkware/solidity/components/FactRegistry.sol /app/build/Release/src/starkware/solidity/components/solidity_contracts_components_sol/starkware/solidity/components/FactRegistry.sol

src/starkware/solidity/components/solidity_contracts_components_sol/starkware/solidity/components/GovernedFinalizable.sol: ../../src/starkware/solidity/components/GovernedFinalizable.sol
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --blue --bold --progress-dir=/app/build/Release/CMakeFiles --progress-num=$(CMAKE_PROGRESS_3) "Copying file GovernedFinalizable.sol"
	cd /app/build/Release/src/starkware/solidity/components && /usr/local/lib/python3.7/site-packages/cmake/data/bin/cmake -E copy /app/src/starkware/solidity/components/GovernedFinalizable.sol /app/build/Release/src/starkware/solidity/components/solidity_contracts_components_sol/starkware/solidity/components/GovernedFinalizable.sol

solidity_contracts_components_sol_copy_files: src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_copy_files
solidity_contracts_components_sol_copy_files: src/starkware/solidity/components/solidity_contracts_components_sol/starkware/solidity/components/FactRegistry.sol
solidity_contracts_components_sol_copy_files: src/starkware/solidity/components/solidity_contracts_components_sol/starkware/solidity/components/GovernedFinalizable.sol
solidity_contracts_components_sol_copy_files: src/starkware/solidity/components/solidity_contracts_components_sol_copy_files.stamp
solidity_contracts_components_sol_copy_files: src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_copy_files.dir/build.make
.PHONY : solidity_contracts_components_sol_copy_files

# Rule to build all files generated by this target.
src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_copy_files.dir/build: solidity_contracts_components_sol_copy_files
.PHONY : src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_copy_files.dir/build

src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_copy_files.dir/clean:
	cd /app/build/Release/src/starkware/solidity/components && $(CMAKE_COMMAND) -P CMakeFiles/solidity_contracts_components_sol_copy_files.dir/cmake_clean.cmake
.PHONY : src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_copy_files.dir/clean

src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_copy_files.dir/depend:
	cd /app/build/Release && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /app /app/src/starkware/solidity/components /app/build/Release /app/build/Release/src/starkware/solidity/components /app/build/Release/src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_copy_files.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : src/starkware/solidity/components/CMakeFiles/solidity_contracts_components_sol_copy_files.dir/depend

